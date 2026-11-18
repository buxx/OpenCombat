use crate::states::GuiFiles;
use bevy::prelude::*;
use oc_root::Wcfg;
use oc_utils::let_some;

use crate::{
    entity::world::{VisibleBattleSquare, WorldMapBackground, minimap::Minimap},
    ingame::{
        camera::{map::world_map_point_to_bevy_world_point, move_::UpdateVisibleBattleSquare},
        draw::{
            Z_WORLD_MAP_BACKGROUND, Z_WORLD_MAP_BATTLE_SQUARE, Z_WORLD_MAP_MINIMAP,
            world::WorldMapDisplay,
        },
    },
};

#[derive(Debug, Event)]
pub struct SpawnMinimap;

#[derive(Debug, Event)]
pub struct AdjustMinimap;

#[derive(Debug, Event)]
pub struct SpawnVisibleBattleSquare;

#[derive(Debug, Event)]
pub struct SpawnWorldMapBackground;

#[derive(Debug, Event)]
pub struct DespawnWorldMapBackground;

pub fn on_spawn_minimap(
    _: On<SpawnMinimap>,
    mut commands: Commands,
    w: Res<Wcfg>,
    window: Single<&Window>,
    assets: Res<AssetServer>,
    files: Res<GuiFiles>,
) {
    let_some!(w = &w.0, return);
    let_some!(files = &files.0, return);

    let display = WorldMapDisplay::from_env(w, window.size());
    let minimap = files.asset(oc_root::files::File::Minimap);

    let x = display.center.x;
    let y = display.center.y;
    let scale_x = display.size.x / w.minimap_width_pixels() as f32;
    let scale_y = display.size.y / w.minimap_height_pixels() as f32;
    commands.spawn((
        Minimap,
        Sprite::from_image(assets.load(minimap)),
        Transform {
            scale: Vec3::new(scale_x, scale_y, 1.0),
            translation: Vec3::new(x, y, Z_WORLD_MAP_MINIMAP),
            ..default()
        },
    ));
}

pub fn on_adjust_minimap(
    _: On<AdjustMinimap>,
    w: Res<Wcfg>,
    mut minimap: Single<&mut Transform, With<Minimap>>,
    window: Single<&Window>,
) {
    let_some!(w = &w.0, return);
    tracing::debug!("Adjust world minimap");
    let display = WorldMapDisplay::from_env(w, window.size());
    let x = display.center.x;
    let y = display.center.y;
    let scale_x = display.size.x / w.minimap_width_pixels() as f32;
    let scale_y = display.size.y / w.minimap_height_pixels() as f32;

    minimap.translation.x = x;
    minimap.translation.y = y;
    minimap.scale.x = scale_x;
    minimap.scale.y = scale_y;
}

pub fn on_spawn_visible_battle_square(
    _: On<SpawnVisibleBattleSquare>,
    mut commands: Commands,
    w: Res<Wcfg>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let_some!(w = &w.0, return);
    let display = WorldMapDisplay::from_env(w, window.size());

    commands.spawn((
        VisibleBattleSquare,
        Transform::from_xyz(display.start.x, display.start.y, Z_WORLD_MAP_BATTLE_SQUARE),
        Mesh2d(meshes.add(Rectangle::new(0.0, 0.0).to_ring(1.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
    ));
}

pub fn on_spawn_world_map_background(
    _: On<SpawnWorldMapBackground>,
    mut commands: Commands,
    w: Res<Wcfg>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let_some!(w = &w.0, return);
    let display = WorldMapDisplay::from_env(w, window.size());

    commands.spawn((
        WorldMapBackground,
        Transform::from_xyz(display.center.x, display.center.y, Z_WORLD_MAP_BACKGROUND),
        Mesh2d(meshes.add(Rectangle::new(display.size.x, display.size.y))),
        MeshMaterial2d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
    ));
}

pub fn on_despawn_world_map_background(
    _: On<DespawnWorldMapBackground>,
    mut commands: Commands,
    query: Single<Entity, With<WorldMapBackground>>,
) {
    commands.entity(query.into_inner()).despawn();
}

pub fn on_update_battle_square(
    center: On<UpdateVisibleBattleSquare>,
    w: Res<Wcfg>,
    window: Single<&Window>,
    square: Single<(&mut Transform, &mut Mesh2d), With<VisibleBattleSquare>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let_some!(w = &w.0, return);
    let (mut transform, mut mesh) = square.into_inner();
    let display = WorldMapDisplay::from_env(w, window.size());
    let point = world_map_point_to_bevy_world_point(w, center.0, window.size());

    transform.translation.x = point.x;
    transform.translation.y = point.y;

    let size = window.size() * display.ratio;
    mesh.0 = meshes.add(Rectangle::new(size.x, size.y).to_ring(1.0));
}
