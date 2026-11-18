use bevy::{image::TextureAtlasLayout, math::UVec2};
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_root::files::Files;
use oc_world::{terrain::Terrain, tile::Tile};
use rustc_hash::FxHashMap;

use crate::{ingame::draw::Z_TERRAIN_TILE, world::World};

#[cfg(feature = "debug")]
impl super::Tileset<WorldTileIndex, Tile> for Terrain {
    fn layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(
            UVec2::splat(self.w.geo_pixels_per_tile() as u32),
            self.columns(),
            self.rows(),
            None,
            None,
        )
    }

    fn spriteset(&self, files: &Files) -> String {
        files.terrain_png()
    }

    fn z(&self) -> f32 {
        Z_TERRAIN_TILE
    }

    fn tiles<'a>(
        &self,
        world: &'a World,
        region: WorldRegionIndex,
    ) -> Option<&'a FxHashMap<WorldTileIndex, Tile>> {
        world.tiles().get(&region)
    }

    fn index(&self, tile: &Tile) -> Option<usize> {
        self.natures.get(&tile.nature).map(|v| (*v) as usize)
    }
}
