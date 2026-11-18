use bevy::image::TextureAtlasLayout;
use oc_geo::region::WorldRegionIndex;
use oc_root::files::Files;
use rustc_hash::FxHashMap;

use crate::world::World;

pub mod height;
pub mod terrain;

pub trait ConcernedTileset<I, T, S: Tileset<I, T>> {
    fn tileset<'a>(&self, world: &'a World) -> &'a Option<S>;
}

pub trait Tileset<I, T> {
    fn layout(&self) -> TextureAtlasLayout;
    fn spriteset(&self, files: &Files) -> String;
    fn z(&self) -> f32;
    fn tiles<'a>(
        &'a self,
        world: &'a World,
        region: WorldRegionIndex,
    ) -> Option<&'a FxHashMap<I, T>>;
    fn index(&self, tile: &T) -> Option<usize>;
}
