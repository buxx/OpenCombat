use bevy::prelude::*;
use oc_root::WorldConfig;

use crate::ingame::{camera, draw::world::WorldMapDisplay};

#[derive(Debug, Event)]
pub struct SaveCurrentWindowCenterAsBattleCenter;

pub fn on_save_current_window_center_as_battle_center(
    _: On<SaveCurrentWindowCenterAsBattleCenter>,
    camera: Single<&mut Transform, With<Camera2d>>,
    mut state: ResMut<camera::State>,
) {
    let point = camera.translation;
    tracing::debug!("Save {point:?} as battle center");
    state.previously = Some(point);
}

pub fn window_point_to_world_map_point(w: &WorldConfig, point: Vec2, window: Vec2) -> Vec2 {
    let display = WorldMapDisplay::from_env(w, window);
    let point = Vec2::new(point.x, display.size.y - point.y); // Invert y (as bevy y way)
    let point = point - display.padding;
    let ratio = point / display.size;
    ratio
        * Vec2::new(
            w.world_width_pixels() as f32,
            w.world_height_pixels() as f32,
        )
}

pub fn world_map_point_to_bevy_world_point(w: &WorldConfig, point: Vec2, window: Vec2) -> Vec2 {
    let display = WorldMapDisplay::from_env(w, window);
    let ratio = point
        / Vec2::new(
            w.world_width_pixels() as f32,
            w.world_height_pixels() as f32,
        );
    let point = ratio * display.size;
    Vec2::new(point.x + display.start.x, point.y + display.start.y)
}
