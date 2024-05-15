use battle_core::{
    map::terrain::{TerrainTile, TileType},
    types::GridPoint,
};

use super::MapModel;

pub struct Flat;

impl MapModel for Flat {
    fn terrain_tile_size(&self) -> u32 {
        5
    }

    fn terrain_tiles(
        &self,
        width: u32,
        height: u32,
        default_tile_type: TileType,
        placed: &[(GridPoint, TileType)],
    ) -> Vec<battle_core::map::terrain::TerrainTile> {
        let mut terrain_tiles = vec![];
        let terrain_tile_size: u32 = 5;
        let columns = width / terrain_tile_size;
        let lines = height / terrain_tile_size;

        for line in 0..lines {
            for column in 0..columns {
                let tile_x = column; // TODO: not sure at all here ...
                let tile_y = line; // TODO: not sure at all here ...

                let tile_type = placed
                    .iter()
                    .find(|x| x.0 == GridPoint::new(column as i32, line as i32))
                    .map(|x| x.1.clone())
                    .unwrap_or(default_tile_type.clone());

                terrain_tiles.push(TerrainTile::new(
                    tile_type,
                    terrain_tile_size,
                    terrain_tile_size,
                    1.0,
                    1.0,
                    column,
                    line,
                    tile_x,
                    tile_y,
                ))
            }
        }

        terrain_tiles
    }
}
