use battle_core::{
    map::terrain::{TerrainTile, TileType},
    types::GridPoint,
};

pub mod flat;
pub mod generator;

pub trait MapModel {
    fn terrain_tiles(
        &self,
        width: u32,
        height: u32,
        default_tile_type: TileType,
        placed: &[(GridPoint, TileType)],
    ) -> Vec<TerrainTile>;
    fn terrain_tile_size(&self) -> u32;
}
