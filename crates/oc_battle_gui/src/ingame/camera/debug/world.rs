use bevy::prelude::*;
use oc_root::Wcfg;
use oc_utils::let_some;

use crate::ingame::{
    camera::map::{window_point_to_world_map_point, world_map_point_to_bevy_world_point},
    draw::{self, world::WorldMapDisplay},
};

#[derive(Debug, Component)]
pub struct WorldCursor;

pub fn setup(
    mut commands: Commands,
    w: Res<Wcfg>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let_some!(w = &w.0, return);
    let display = WorldMapDisplay::from_env(w, window.size());
    commands.spawn((
        WorldCursor,
        Transform::from_xyz(display.start.x, display.start.y, draw::Z_WORLD_CURSOR),
        Mesh2d(meshes.add(Circle::new(4.0))),
        MeshMaterial2d(materials.add(Color::srgb(0., 0., 1.))),
    ));
}

pub fn cursor(
    window: Single<&Window>,
    w: Res<Wcfg>,
    mut cursor: Single<&mut Transform, With<WorldCursor>>,
) {
    let_some!(w = &w.0, return);
    let_some!(point = window.cursor_position(), return);

    // We do the compute in way then in opposite way to test code
    let point = window_point_to_world_map_point(w, point, window.size());
    let point = world_map_point_to_bevy_world_point(w, point, window.size());

    cursor.translation.x = point.x;
    cursor.translation.y = point.y;
}
