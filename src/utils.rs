use std::io::Read;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer};
use serde_json::{Number, Value};

use crate::error::TiledError;

/// Algoritm used to compress the tile layer data.
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    Zlib,
    Gzip,
}

/// Encoding used to encode the tile layer data.
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoding {
    Csv,
    Base64,
}

pub fn decode_zlib(data: Vec<u8>) -> Result<Vec<u8>, TiledError> {
    use libflate::zlib::Decoder;
    let mut buffer = Vec::new();

    Decoder::new(&data[..])
        .and_then(|mut decoder| decoder.read_to_end(&mut buffer))
        .map_err(TiledError::DecompressingError)?;

    Ok(buffer)
}

pub fn decode_gzip(data: Vec<u8>) -> Result<Vec<u8>, TiledError> {
    use libflate::gzip::Decoder;
    let mut buffer = Vec::new();

    Decoder::new(&data[..])
        .and_then(|mut decoder| decoder.read_to_end(&mut buffer))
        .map_err(TiledError::DecompressingError)?;

    Ok(buffer)
}

pub fn decode_tiledata(
    data: Value,
    width: u32,
    height: u32,
    encoding: Option<Encoding>,
    compression: Option<Compression>,
) -> Result<Vec<u32>, TiledError> {
    use std::convert::TryFrom;

    // Pre allocate space for all tiles.
    let mut tiles = Vec::with_capacity(usize::try_from(width * height).unwrap_or(0));

    match encoding {
        Some(Encoding::Base64) => decode_base64_tiledata(data, compression, &mut tiles),
        Some(Encoding::Csv) | None => decode_csv_tiledata(data, &mut tiles),
    }?;

    Ok(tiles)
}

/// Decode base64 encoded (possibly compressed) data.
pub fn decode_base64_tiledata(
    data: Value,
    compression: Option<Compression>,
    tiles: &mut Vec<u32>,
) -> Result<(), TiledError> {
    use std::convert::TryInto;

    let data = data
        .as_str()
        .ok_or_else(|| TiledError::Other("Improperly formatted data".to_string()))?;

    let bytes = base64::decode(data.trim().as_bytes()).map_err(TiledError::Base64DecodingError)?;

    let bytes = match compression {
        Some(Compression::Gzip) => decode_gzip(bytes),
        Some(Compression::Zlib) => decode_zlib(bytes),
        None => Ok(bytes),
    }?;

    // Read u32s from buffer into 1d vec of u32.
    for chunk in bytes.chunks(std::mem::size_of::<u32>()) {
        let tile_value = u32::from_le_bytes(
            chunk
                .try_into()
                .map_err(|err| TiledError::Other(format!("{:?}", err)))?,
        );

        tiles.push(tile_value);
    }

    Ok(())
}

/// Decode csv encoded data (default is csv).
pub fn decode_csv_tiledata(data: Value, tiles: &mut Vec<u32>) -> Result<(), TiledError> {
    // Tiledata is stored in array of numbers.
    if let Some(data) = data.as_array() {
        for value in data {
            // Ignore all non-number values.
            if let Some(value) = value.as_u64() {
                tiles.push(value as u32);
            }
        }

        Ok(())
    } else {
        Err(TiledError::Other("Improperly formatted data".to_string()))
    }
}

/// Deserialize map version number from json number to string.
/// This function could also signal error if version number is not supported.
pub fn deserialize_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let version: Number = Deserialize::deserialize(deserializer)?;
    Ok(version.to_string())
}

/// Color as rgba.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Color([u8; 4]);

/// Convert hex string to rgb bytes.
impl FromStr for Color {
    type Err = TiledError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = if s.starts_with('#') { &s[1..] } else { s };
        let mut color = [std::u8::MAX; 4];

        // Tiled colors are ither #rrggbb or #aarrggbb ('#' is trimmed above).
        if s.len() != 6 && s.len() != 8 {
            return Err(TiledError::Other(format!("Invalid color value {:?}", s)));
        }

        // Read two characters to u8 with u8::from_str_radix(..., 16).
        for i in (0..s.len()).step_by(2) {
            color[i / 2] = u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|_| TiledError::Other(format!("Invalid color value {:?}", s)))?;
        }

        if s.len() == 8 {
            // Swap alpha channel to last byte (argb -> rgba).
            Ok(Color {
                0: [color[1], color[2], color[3], color[0]],
            })
        } else {
            Ok(Color { 0: color })
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Color::from_str(&(Deserialize::deserialize(deserializer) as Result<String, _>)?)
            .map_err(de::Error::custom)
    }
}
