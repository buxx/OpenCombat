use battle_core::map::terrain::{TerrainTile, TileType};

use super::MapModel;

pub struct FlatAndEmpty;

impl MapModel for FlatAndEmpty {
    fn terrain_tile_size(&self) -> u32 {
        5
    }

    fn terrain_tiles(
        &self,
        width: u32,
        height: u32,
    ) -> Vec<battle_core::map::terrain::TerrainTile> {
        let mut terrain_tiles = vec![];
        let terrain_tile_size: u32 = 5;
        let columns = width / terrain_tile_size;
        let lines = height / terrain_tile_size;

        for x in 0..lines {
            for y in 0..columns {
                let tile_x = x; // TODO: not sure at all here ...
                let tile_y = y; // TODO: not sure at all here ...
                terrain_tiles.push(TerrainTile::new(
                    TileType::ShortGrass,
                    terrain_tile_size,
                    terrain_tile_size,
                    1.0,
                    1.0,
                    x,
                    y,
                    tile_x,
                    tile_y,
                ))
            }
        }

        terrain_tiles
    }
}
