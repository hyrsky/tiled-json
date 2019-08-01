use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{de::Error, Deserialize, Deserializer};
use serde_json::Value;

mod error;
mod properties;
mod tileset;
mod utils;

use crate::properties::deserialize_properties;
use crate::utils::{decode_tiledata, deserialize_version, Compression, Encoding};

pub use crate::error::TiledError;
pub use crate::properties::{Properties, Property};
pub use crate::tileset::Tileset;
pub use crate::utils::Color;

/// Tile orientation.
#[derive(Debug, PartialEq, Eq, Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Text {
    text: String,
    wrap: bool,

    #[serde(rename = "fontfamily")]
    font_family: Option<String>,

    #[serde(rename = "pixelsize")]
    pixel_size: Option<u32>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
enum ObjectShapeData {
    Point {
        point: bool,
    },
    Ellipse {
        ellipse: bool,
        width: f32,
        height: f32,
    },
    Polyline {
        #[serde(rename = "polyline")]
        points: Vec<Point>,
    },
    Polygon {
        #[serde(rename = "polygon")]
        points: Vec<Point>,
    },
    Text {
        text: Text,
        width: f32,
        height: f32,
    },
    Rect {
        width: f32,
        height: f32,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum ObjectShape {
    Point,
    Rect { width: f32, height: f32 },
    Ellipse { width: f32, height: f32 },
    Polyline { points: Vec<Point> },
    Polygon { points: Vec<Point> },
    Text { text: Text, width: f32, height: f32 },
    Unknown,
}

impl ObjectShape {
    fn from(data: ObjectShapeData) -> Self {
        match data {
            ObjectShapeData::Point { .. } => ObjectShape::Point,
            ObjectShapeData::Rect { width, height } => ObjectShape::Rect { width, height },
            ObjectShapeData::Ellipse { width, height, .. } => {
                ObjectShape::Ellipse { width, height }
            }
            ObjectShapeData::Polyline { points } => ObjectShape::Polyline { points },
            ObjectShapeData::Polygon { points } => ObjectShape::Polygon { points },
            ObjectShapeData::Text {
                width,
                height,
                text,
            } => ObjectShape::Text {
                width,
                height,
                text,
            },
        }
    }
}

impl<'de> Deserialize<'de> for ObjectShape {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Ok(shape_data) = Deserialize::deserialize(deserializer) {
            return Ok(ObjectShape::from(shape_data));
        }

        Ok(ObjectShape::Unknown)
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Object {
    /// Unique ID of the object. Each object that is placed on a map gets a unique id.
    pub id: u32,
    /// The name of the object. An arbitrary string.
    pub name: String,
    /// The type of the object. An arbitrary string.
    pub r#type: String,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub visible: bool,

    #[serde(flatten)]
    pub shape: ObjectShape,

    /// Custom properties
    #[serde(default, deserialize_with = "deserialize_properties")]
    pub properties: Option<Properties>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ObjectGroup {
    pub objects: Vec<Object>,
    pub color: Option<Color>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ImageLayer {
    #[serde(rename = "offsetx")]
    pub offset_x: f32,
    #[serde(rename = "offsety")]
    pub offset_y: f32,
    #[serde(rename = "transparentcolor")]
    pub transparent_color: Option<Color>,

    pub image: String,
}

/// Internal type that deserializes from tiled json format.
#[derive(Debug, PartialEq, Clone, Deserialize)]
struct TileLayerData {
    /// Column count. Same as map width for fixed-size maps.
    width: u32,
    /// Row count. Same as map height for fixed-size maps.
    height: u32,
    /// Type of data depends on encoding.
    data: Value,
    compression: Option<Compression>,
    encoding: Option<Encoding>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TileLayer {
    /// Column count. Same as map width for fixed-size maps.
    width: u32,
    /// Row count. Same as map height for fixed-size maps.
    height: u32,
    /// Tiles arranged in a 1d array.
    tiles: Vec<u32>,
}

impl TileLayer {
    /// Construct TileLayer from TileLayerData.
    fn from(layer_data: TileLayerData) -> Result<Self, TiledError> {
        Ok(TileLayer {
            width: layer_data.width,
            height: layer_data.height,
            tiles: decode_tiledata(
                layer_data.data,
                layer_data.width,
                layer_data.height,
                layer_data.encoding,
                layer_data.compression,
            )?,
        })
    }

    /// Get tile with x and y coordinates.
    /// This is equivalent to `layer.tiles[x + y * layer.width]`
    pub fn get_tile(&self, x: u32, y: u32) -> u32 {
        self.tiles[(x + y * self.width) as usize]
    }
}

impl<'de> Deserialize<'de> for TileLayer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize to intermediary struct TileLayerData to allow
        // decompressing and decoding tile data.
        TileLayer::from(Deserialize::deserialize(deserializer)?).map_err(Error::custom)
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum LayerType {
    TileLayer(TileLayer),
    ImageLayer(ImageLayer),
    ObjectGroup(ObjectGroup),
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Layer {
    /// The name of the layer.
    pub name: String,
    /// The opacity of the layer as a value from 0 to 1. Defaults to 1.
    pub opacity: f32,
    /// Whether the layer is shown or hidden.
    pub visible: bool,

    /// Layer data depends on layer type.
    #[serde(flatten)]
    pub data: LayerType,

    /// Custom properties
    #[serde(default, deserialize_with = "deserialize_properties")]
    pub properties: Option<Properties>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Map {
    /// File format version
    #[serde(deserialize_with = "deserialize_version")]
    pub version: String,
    pub orientation: Orientation,
    /// Number of tile columns
    pub width: u32,
    /// Number of tile rows
    pub height: u32,
    /// Map grid width
    #[serde(rename = "tilewidth")]
    pub tile_width: u32,
    /// Map grid height
    #[serde(rename = "tileheight")]
    pub tile_height: u32,
    pub tilesets: Vec<Tileset>,
    pub layers: Vec<Layer>,
    #[serde(rename = "backgroundcolor")]
    pub background_colour: Option<Color>,
    /// Custom properties
    #[serde(default, deserialize_with = "deserialize_properties")]
    pub properties: Option<Properties>,
}

/// Read buffer hopefully containing a Tiled map and try to parse it.
pub fn parse<R: Read>(reader: R) -> Result<Map, TiledError> {
    serde_json::from_reader(reader).map_err(TiledError::ParsingError)
}

/// Read file hopefully containing a Tiled map and try to parse it.
pub fn parse_file(path: &Path) -> Result<Map, TiledError> {
    let file = File::open(path).map_err(|err| TiledError::Other(format!("{:?}", err)))?;

    parse(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_objects() {
        let map = parse_file(&Path::new("assets/map.json")).unwrap();

        // Map file should contain non-empty object layer.
        map.layers
            .iter()
            .find(|layer| match &layer.data {
                LayerType::ObjectGroup(group) => !group.objects.is_empty(),
                _ => false,
            })
            .unwrap();
    }

    #[allow(clippy::approx_constant)]
    #[test]
    fn test_properties() {
        let map = parse_file(&Path::new("assets/map.json")).unwrap();

        // Map file should contain properties.
        let properties = map.properties.unwrap();

        assert_eq!(properties.get("pi").unwrap(), &Property::Float(3.14));
        assert_eq!(properties.get("answer").unwrap(), &Property::Int(42));
    }

    #[test]
    fn test_encodings() {
        let a = parse_file(&Path::new("assets/map.json")).unwrap();
        let b = parse_file(&Path::new("assets/map_csv.json")).unwrap();

        // Map file should contain properties.
        assert_eq!(a.layers.len(), b.layers.len());

        for i in 0..a.layers.len() {
            if let (LayerType::TileLayer(a), LayerType::TileLayer(b)) =
                (&a.layers[i].data, &b.layers[i].data)
            {
                assert_eq!(&a.tiles[..], &b.tiles[..]);
            }
        }
    }
}
