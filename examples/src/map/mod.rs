use battle_core::map::terrain::TerrainTile;

pub mod flat;
pub mod generator;

pub trait MapModel {
    fn terrain_tiles(&self, width: u32, height: u32) -> Vec<TerrainTile>;
    fn terrain_tile_size(&self) -> u32;
}
