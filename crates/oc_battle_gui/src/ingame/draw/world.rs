use bevy::prelude::*;
use oc_root::WorldConfig;

const WORLD_MAP_OFFSET_X: f32 = -100_000.;
const WORLD_MAP_OFFSET_Y: f32 = -100_000.;
const WORLD_MAP_WINDOW_PADDING: f32 = 10.0;

pub struct WorldMapDisplay {
    pub start: Vec2,
    pub size: Vec2,
    pub center: Vec2,
    pub padding: Vec2,
    pub ratio: Vec2,
}

impl WorldMapDisplay {
    pub fn from_env(w: &WorldConfig, window: Vec2) -> Self {
        let start = Vec2::new(WORLD_MAP_OFFSET_X, WORLD_MAP_OFFSET_Y) + WORLD_MAP_WINDOW_PADDING;
        let size = window - (WORLD_MAP_WINDOW_PADDING * 2.);
        let center = start + (size / 2.);
        let padding = Vec2::new(WORLD_MAP_WINDOW_PADDING, -WORLD_MAP_WINDOW_PADDING);
        let world = Vec2::new(
            w.world_width_pixels() as f32,
            w.world_height_pixels() as f32,
        );
        let ratio = size / world;

        Self {
            start,
            size,
            center,
            padding,
            ratio,
        }
    }
}
