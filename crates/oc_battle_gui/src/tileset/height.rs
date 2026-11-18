use bevy::{image::TextureAtlasLayout, math::UVec2};
use oc_geo::{region::WorldRegionIndex, tile::WorldHeightIndex};
use oc_root::files::Files;
use oc_world::terrain::Terrain;
use rustc_hash::FxHashMap;

use crate::{ingame::draw::Z_TERRAIN_TILE, world::World};

#[cfg(feature = "debug")]
impl super::Tileset<WorldHeightIndex, u8> for Terrain {
    fn layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(
            UVec2::splat(self.w.geo_pixels_per_tile() as u32),
            10, // FIXME: data from real terrain txs
            10, // FIXME: data from real terrain txs
            None,
            None,
        )
    }

    fn spriteset(&self, files: &Files) -> String {
        files.height_png()
    }

    fn z(&self) -> f32 {
        Z_TERRAIN_TILE
    }

    fn tiles<'a>(
        &self,
        world: &'a World,
        region: WorldRegionIndex,
    ) -> Option<&'a FxHashMap<WorldHeightIndex, u8>> {
        world.heights().get(&region)
    }

    fn index(&self, height: &u8) -> Option<usize> {
        Some((*height) as usize)
    }
}
