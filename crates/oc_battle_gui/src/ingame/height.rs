use std::f32::consts::PI;

use bevy::color::palettes::css::WHITE;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy_heightmap::{HeightMap, HeightMapPlugin, ValueFunctionHeightMap};
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_physics::Physic;
use oc_physics::volume::Volume;
use oc_root::files::File;
use oc_root::y::{V, Y};
use oc_root::{Wcfg, WcfgFrom, WcfgInto, WorldConfig};
use oc_utils::d2::Xy;
use oc_utils::{let_ok, let_some};

use crate::ingame::InGameState;
use crate::states::{AppState, GameConfig};
use crate::world::World;

// WARNING: this module is experimental and essentially based on AI help

pub struct HeightPlugin;

#[derive(Event)]
pub struct Spawn(pub Vec2);

#[derive(Component, Default)]
struct Height {}

/// Marker for the mouse cursor indicator circle
#[derive(Component)]
pub struct CursorCircle;

/// Resource holding the current world-space cursor position
#[derive(Resource, Default)]
pub struct CursorWorldPos(pub Option<Vec3>);

impl Plugin for HeightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((HeightMapPlugin, MeshPickingPlugin))
            .add_observer(on_spawn)
            .add_systems(
                Update,
                (
                    control_camera3d,
                    update_cursor_world_pos,
                    update_cursor_circle_transform,
                )
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Height)),
            );
    }
}

// FIXME: if all tiles are z0, nothing display
// FIXME: avoid blocking way of background + mesh generation by doing it in background and before real need
#[allow(clippy::too_many_arguments)]
fn on_spawn(
    center: On<Spawn>,
    g: Res<GameConfig>,
    files: Res<crate::states::GuiFiles>,
    world: Res<World>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    tracing::debug!("Spawn height map");
    let_some!(g = &g.0, return);
    let_some!(files = &files.0, return);
    let center = center.0;
    tracing::trace!(name="ingame-height-on-spawn-center", center=?center);

    let regions: Vec<WorldRegionIndex> = world.tiles.keys().cloned().collect();
    let grid_size = UVec2::new(g.w.region_width() as u32, g.w.region_height() as u32);

    for region in regions {
        let background = File::Region {
            index: region.0,
            width: g.w.region_width(),
            height: g.w.region_height(),
        };
        if !files.fs(background).exists() {
            tracing::warn!(
                "Can't build heigh map for region {region:?}: background file {} do not exist",
                files.fs(background).display()
            );
            continue;
        }
        let texture: Handle<Image> = asset_server.load(files.asset(background));
        tracing::trace!(name = "ingame-height-on-spawn-mesh-generate", region = ?region);

        let Some(tiles) = world.tiles.get(&region) else {
            tracing::warn!("Can't build heigh map for region {region:?}: no known tiles",);
            continue;
        };
        let z_max = tiles.values().map(|tile| tile.z).max().unwrap_or_default();
        let mesh: Handle<Mesh> = meshes.add(
            ValueFunctionHeightMap(|p: Vec2| {
                // p is given as retlative like (top-left) -0.5,-0.5, (center) 0.0,0.0, etc.
                // So, add 0.5 to have something relative from 0.0 to 1.0,
                // then, * region_width/ieght to find point is.
                let p_ = Vec2::new(
                    (p.x + 0.5) * g.w.region_width() as f32,
                    (p.y + 0.5) * g.w.region_height() as f32,
                );
                // Remove region_height to adapt to inverted y
                let (x, y) = (p_.x as u64, (g.w.region_height() as f32 - p_.y) as u64);
                let tile = TileXy(Xy(x, y));
                let tile_i: WorldTileIndex = tile.into_(&g.w);

                tiles
                    .get(&tile_i)
                    .map(|tile| tile.z as f32 / z_max as f32)
                    .unwrap_or_default()
            })
            .build_mesh(grid_size),
        );

        tracing::trace!(name = "ingame-height-on-spawn-spawn");

        let width = g.w.region_width_pixels() as f32;
        let height = g.w.region_height_pixels() as f32;

        let region_: RegionXy = region.into_(&g.w);
        let x = region_.0.0 as f32 * width;
        let y = region_.0.1 as f32 * height;
        let x = x + width / 2.;
        let y = y + height / 2.;
        let y = y.to_gui_y(&g.w);

        commands.spawn((
            Height::default(),
            Mesh3d(mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(texture),
                perceptual_roughness: 1.0,
                metallic: 0.0,
                reflectance: 0.0,
                specular_transmission: 0.0,
                ..default()
            })),
            Transform {
                translation: Vec3::new(x, y, 0.),
                scale: Vec3::new(
                    width,
                    height,
                    z_max as f32 * g.w.geo_meters_per_z().0 * g.w.geo_pixels_per_meters(),
                ),
                // .extend(z_max as f32 * w.geo_meters_per_z.0 * w.geo_pixels_per_meters),
                ..default()
            },
        ));

        // tiles cubes
        tracing::trace!(name = "ingame-height-on-spawn-tiles");
        for (i, tile) in tiles {
            let nature = g.mod_.nature(tile.nature);
            let nature_z = nature.z.0;

            if nature_z == 0.0 {
                continue;
            }

            let xy = TileXy::from_(*i, &g.w);
            let (x, y) = (
                xy.0.0 * g.w.geo_pixels_per_tile() + g.w.geo_pixels_per_tile() / 2,
                xy.0.1 * g.w.geo_pixels_per_tile() + g.w.geo_pixels_per_tile() / 2,
            );
            let z = tile.z_pixels(&g.w);
            let alpha = nature.opacity.min(0.5);
            let x_length = g.w.geo_pixels_per_tile() as f32;
            let y_length = g.w.geo_pixels_per_tile() as f32;
            let z_length = nature_z * g.w.geo_pixels_per_meters();
            let x = x as f32;
            let y = (y as f32).to_gui_y(&g.w);
            tracing::trace!(
                name = "ingame-height-on-spawn-tiles-tile",
                x_length = x_length,
                y_length = y_length,
                z_length = z_length,
                x = x,
                y = y,
                z = z
            );
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(x_length, y_length, z_length))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.5, 1.0, 0.5, alpha),
                    alpha_mode: AlphaMode::AlphaToCoverage,
                    ..default()
                })),
                Transform::from_xyz(x, y, z),
            ));
        }

        // individuals cube
        tracing::trace!(name = "ingame-height-on-spawn-individuals");
        if let Some(individuals) = world.individuals.get(&region) {
            for (tile, individuals) in individuals {
                for (_, individual) in individuals {
                    if let Some(tile) = tiles.get(tile) {
                        let tile_xy = TileXy::from_(tile.i, &g.w);
                        let x = tile_xy.0.0 as f32 * g.w.geo_pixels_per_tile() as f32;
                        let y = tile_xy.0.1 as f32 * g.w.geo_pixels_per_tile() as f32;
                        let y = y.to_gui_y(&g.w);
                        let z = tile.z_pixels(&g.w);
                        let ref_ = [x, y, z].into();
                        let rotation = individual.gesture.rotation(V::Gui);
                        for (volume, _, _) in individual.volumes(ref_, false, &g.w, &g.mod_) {
                            let alpha = 1.0;
                            if let Volume::Cube {
                                x,
                                y,
                                z,
                                width,
                                height,
                                depth,
                            } = volume
                            {
                                let x_length = width;
                                let y_length = height;
                                let z_length = depth;
                                let x = x + width / 2.0;
                                let y = y + height / 2.0;
                                let z = z + depth;
                                tracing::trace!(
                                    name = "ingame-height-on-spawn-individuals-individual",
                                    x_length = x_length,
                                    y_length = y_length,
                                    z_length = z_length,
                                    x = x,
                                    y = y,
                                    z = z
                                );
                                commands.spawn((
                                    Mesh3d(meshes.add(Cuboid::new(width, height, depth))),
                                    MeshMaterial3d(materials.add(StandardMaterial {
                                        base_color: Color::srgba(0.0, 0.0, 1.0, alpha),
                                        alpha_mode: AlphaMode::AlphaToCoverage,
                                        ..default()
                                    })),
                                    Transform::from_xyz(x, y, z).with_rotation(rotation),
                                ));
                            }
                        }
                    }
                }
            }
        };
    }

    //
    let mesh = meshes.add(Annulus::new(0.9, 1.0).mesh().resolution(64).build());

    commands.spawn((
        CursorCircle,
        Mesh3d(mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, 0.8),
            alpha_mode: AlphaMode::Blend,
            unlit: true, // ignore lighting so it's always visible
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden, // hide until we have a valid hit
    ));

    commands.insert_resource(CursorWorldPos::default());
}
fn update_cursor_world_pos(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut cursor_pos: ResMut<CursorWorldPos>,
    w: Res<Wcfg>,
    world: Res<World>,
) {
    let_ok!(window = windows.single(), return);
    let_ok!((camera, cam_transform) = cameras.single(), return);

    let Some(cursor_px) = window.cursor_position() else {
        cursor_pos.0 = None;
        return;
    };

    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_px) else {
        cursor_pos.0 = None;
        return;
    };

    if ray.direction.z.abs() <= 1e-6 {
        cursor_pos.0 = None;
        return;
    }

    // --- Passe 1 : intersection avec Z = 0 pour estimer (x, y) ---
    let t0 = (0.0_f32 - ray.origin.z) / ray.direction.z;
    let approx_hit = ray.origin + ray.direction * t0;

    // --- Récupère la vraie hauteur du terrain à (x, y) ---
    let terrain_z = if let Some(cfg) = &w.0 {
        sample_terrain_z(cfg, &world, &approx_hit)
    } else {
        0.0
    };

    // --- Passe 2 : intersection avec Z = terrain_z ---
    let t1 = (terrain_z - ray.origin.z) / ray.direction.z;
    let hit = ray.origin + ray.direction * t1;

    cursor_pos.0 = Some(hit);
}
fn update_cursor_circle_transform(
    cursor_pos: Res<CursorWorldPos>,
    mut circle: Query<(&mut Transform, &mut Visibility), With<CursorCircle>>,
) {
    let Ok((mut transform, mut visibility)) = circle.single_mut() else {
        return;
    };

    match cursor_pos.0 {
        None => {
            *visibility = Visibility::Hidden;
        }
        Some(pos) => {
            *visibility = Visibility::Inherited;
            let radius = 5.0_f32;

            // pos.z est déjà terrain_z, on ajoute juste l'offset anti z-fight
            transform.translation = Vec3::new(pos.x, pos.y, pos.z + 0.5);
            transform.scale = Vec3::new(radius, radius, 1.0);
        }
    }
}

/// Optional: look up the terrain height at (x, y) the same way on_spawn does.
/// If you don't need the circle to hug the terrain, just return 0.0.
fn sample_terrain_z(w: &WorldConfig, world: &World, pos: &Vec3) -> f32 {
    let (x, y) = (
        (pos.x / w.geo_pixels_per_tile() as f32) as u64,
        (pos.y.to_gui_y(w) / w.geo_pixels_per_tile() as f32) as u64,
    );
    let tile = TileXy(Xy(x, y));
    let tile = WorldTileIndex::from_(tile, w);
    let region = WorldRegionIndex::from_(tile, w);

    world
        .tiles
        .get(&region)
        .and_then(|tiles| {
            tiles
                .get(&tile)
                .map(|tile| tile.z as f32 * w.geo_meters_per_z().0 * w.geo_pixels_per_meters())
        })
        .unwrap_or_default()
}

// WARNING: AI generated to remplace bevy_editor_cam
/// Orthographic camera controls: right drag orbits around the point looked at on the z=0 plane,
/// middle drag pans, wheel zooms.
fn control_camera3d(
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut cameras: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    const ORBIT_SPEED: f32 = 0.005;
    const ZOOM_SPEED: f32 = 0.1;

    for (mut transform, mut projection) in &mut cameras {
        let Projection::Orthographic(ortho) = &mut *projection else {
            continue;
        };

        if scroll.delta.y != 0. {
            ortho.scale = (ortho.scale * (-scroll.delta.y * ZOOM_SPEED).exp()).clamp(0.05, 100.);
        }

        let delta = motion.delta;
        if delta == Vec2::ZERO {
            continue;
        }

        if buttons.pressed(MouseButton::Middle) {
            let offset = *transform.right() * -delta.x + *transform.up() * delta.y;
            transform.translation += offset * ortho.scale;
        } else if buttons.pressed(MouseButton::Right) {
            let forward = transform.forward();
            let pivot = if forward.z.abs() > 1e-4 {
                transform.translation + *forward * (-transform.translation.z / forward.z)
            } else {
                transform.translation
            };
            let yaw = Quat::from_axis_angle(Vec3::Z, -delta.x * ORBIT_SPEED);
            let pitch = Quat::from_axis_angle(*transform.right(), -delta.y * ORBIT_SPEED);
            let rotation = yaw * pitch;
            transform.translation = pivot + rotation * (transform.translation - pivot);
            transform.rotation = rotation * transform.rotation;
        }
    }
}

pub fn setup_camera3d(commands: &mut Commands, center: &Vec2) {
    let mut transform = Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y);
    transform.translation.x = center.x;
    transform.translation.y = center.y;

    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            near: -100.0, // negative near lets you see meshes slightly in front of camera Z
            far: 20000.0,
            ..OrthographicProjection::default_3d()
        }),
        transform,
    ));
}

pub fn setup_light3d(commands: &mut Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 1000.) // original light position
            .with_rotation(Quat::from_axis_angle(Vec3::ONE, -PI / 6.)),
        DirectionalLight {
            color: WHITE.into(),
            illuminance: 4500.,
            shadow_maps_enabled: true,
            ..default()
        },
    ));
}
