use std::path::Path;

use derive_more::Constructor;
use glam::Vec2;
use oc_geo::tile::WorldTileIndex;

/// "On the top" decor (vegetation, ...) displayed on a tile
#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Decor {
    pub i: WorldTileIndex,
    /// Id of the sprite in the decor tileset
    pub tile: u32,
    /// Explosion destruction force required to remove this decor
    pub destruction_resistance: f32,
}

/// Information about the decor tileset required to display `Decor`
#[derive(Debug, Clone)]
pub struct DecorTileset {
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub offset: Vec2,
}

impl DecorTileset {
    pub fn load(path: &Path) -> Result<Self, tiled::Error> {
        let tileset = tiled::Loader::new().load_tsx_tileset(path)?;
        Ok(Self {
            tile_width: tileset.tile_width,
            tile_height: tileset.tile_height,
            columns: tileset.columns,
            rows: tileset.tilecount / tileset.columns.max(1),
            offset: Vec2::new(tileset.offset_x as f32, tileset.offset_y as f32),
        })
    }
}
