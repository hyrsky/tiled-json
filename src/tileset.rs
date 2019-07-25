use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Frame {
	tile_id: u32,
	duration: u32,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Tile {
	/// Local ID of the tile
	pub id: u32,
}

/// A tileset, usually the tilesheet image.
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Tileset {
	/// GID corresponding to the first tile in the set
	#[serde(rename = "firstgid")]
	pub first_gid: u32,
	/// Name given to this tileset
	pub name: String,
	/// Maximum width of tiles in this set
	#[serde(rename = "tilewidth")]
	pub tile_width: u32,
	/// Maximum height of tiles in this set
	#[serde(rename = "tileheight")]
	pub tile_height: u32,
	/// Spacing between adjacent tiles in image (pixels)
	pub spacing: u32,
	/// Buffer between image edge and first tile (pixels)
	pub margin: u32,
	/// Image used for tiles in this set
	pub image: String,
	/// Tileset can associate information with each tile, like its image path
	/// or terrain type.
	pub tiles: Option<Vec<Tile>>,
}
