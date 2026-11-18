use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_root::y::Y;
use oc_root::{Wcfg, WcfgInto};
use oc_utils::let_some;

use super::{ForgottenRegion, ListeningRegion};
use crate::entity::world::region::RegionWireFrame;
use crate::ingame::draw;
use crate::ingame::draw::world::WorldMapDisplay;

#[derive(Debug, Event)]
pub struct SpawnRegionWireFrameDebug(pub WorldRegionIndex);

#[derive(Debug, Event)]
pub struct DespawnRegionWireFrameDebug(pub WorldRegionIndex);

#[derive(Debug, Component)]
pub struct RegionWireFrameDebug;

pub fn on_listening_region(region: On<ListeningRegion>, mut commands: Commands) {
    commands.trigger(SpawnRegionWireFrameDebug(region.0));
}

pub fn on_spawn_region_wire_frame_debug(
    region: On<SpawnRegionWireFrameDebug>,
    mut commands: Commands,
    w: Res<Wcfg>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let_some!(w = &w.0, return);

    // Battle display
    let width = w.region_width() as f32 * w.geo_pixels_per_tile() as f32;
    let height = w.region_height() as f32 * w.geo_pixels_per_tile() as f32;
    let rectangle = Rectangle::new(width, height);
    let rectangle = rectangle.to_ring(1.0);
    let color = Color::srgba(1., 1., 0., 0.5);
    let xy: RegionXy = region.0.into_(w);
    let x = xy.0.0 as f32 * width + width / 2.;
    let y = xy.0.1 as f32 * height + height / 2.;
    let y = y.to_gui_y(w);
    commands.spawn((
        RegionWireFrame(region.0.into_(w)),
        RegionWireFrameDebug,
        Mesh2d(meshes.add(rectangle)),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(x, y, draw::Z_REGION_WIREFRAME),
    ));

    // World display
    let display = WorldMapDisplay::from_env(w, window.size());
    let width = w.region_width_pixels() as f32 * display.ratio.x;
    let height = w.region_height_pixels() as f32 * display.ratio.y;
    let rectangle = Rectangle::new(width, height);
    let rectangle = rectangle.to_ring(1.0);
    let color = Color::srgba(1., 1., 0., 0.5);
    let xy: RegionXy = region.0.into_(w);
    let x = (xy.0.0 as f32 * width + width / 2.) + display.start.x;
    let y = xy.0.1 as f32 * height + height / 2. + display.start.y;
    commands.spawn((
        RegionWireFrame(region.0.into_(w)),
        RegionWireFrameDebug,
        Mesh2d(meshes.add(rectangle)),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(x, y, draw::Z_REGION_WIREFRAME),
    ));
}

pub fn on_forgotten_region(region: On<ForgottenRegion>, mut commands: Commands) {
    commands.trigger(DespawnRegionWireFrameDebug(region.0));
}

pub fn on_despawn_region_wire_frame_debug(
    region: On<DespawnRegionWireFrameDebug>,
    mut commands: Commands,
    w: Res<Wcfg>,
    query: Query<(Entity, &RegionWireFrame), With<RegionWireFrameDebug>>,
) {
    let_some!(w = &w.0, return);
    let region: RegionXy = region.0.into_(w);
    for (entity, region_) in query {
        if region_.0 == region {
            commands.entity(entity).despawn();
        }
    }
}
