use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use geo::TriangulateEarcut;
use oc_root::{SPAWN_ZONE_ENEMY_OPACITY, SPAWN_ZONE_NONE_OPACITY, WorldConfig};
use oc_utils::let_some;
use oc_world::spawn::{SpawnZone, SpawnZoneName};

use crate::{ingame::draw::Z_SPAWN_ZONE, network};

/// Spawn zone owned by the player side
#[derive(Debug, Component)]
pub struct OwnSpawnZone;

#[derive(Debug, Event)]
pub struct ShowSpawnZones(pub Vec<SpawnZoneName>, pub Vec<SpawnZoneName>);

#[derive(Debug, Event)]
pub struct HideSpawnZones;

pub fn spawn_zone(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    w: &WorldConfig,
    spawn_zone: SpawnZone,
) {
    tracing::trace!(name="spawn-spawn-zone", zone=?spawn_zone.name);
    let polygon = spawn_zone.bevy_polygon(w);
    let mesh = polygon_mesh(&polygon);
    let color = Color::srgba(0., 0., 0., 0.);

    commands.spawn((
        Name::new(spawn_zone.name.0.clone()),
        Mesh2d(meshes.add(mesh)),
        MeshMaterial2d(materials.add(color)),
        Transform::from_xyz(0., 0., Z_SPAWN_ZONE),
        Visibility::Hidden,
        spawn_zone,
    ));
}

/// Warning: AI generated function
fn polygon_mesh(polygon: &geo::Polygon<f32>) -> Mesh {
    let triangulation = polygon.earcut_triangles_raw();
    let positions: Vec<[f32; 3]> = triangulation
        .vertices
        .iter()
        .map(|v| [v[0], v[1], 0.])
        .collect();
    let indices: Vec<u32> = triangulation
        .triangle_indices
        .iter()
        .map(|i| *i as u32)
        .collect();

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn on_show_spawn_zones(
    event: On<ShowSpawnZones>,
    mut commands: Commands,
    network: Res<network::state::State>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    spawns: Query<
        (
            Entity,
            &Name,
            &mut Visibility,
            &MeshMaterial2d<ColorMaterial>,
        ),
        With<SpawnZone>,
    >,
) {
    let_some!(identity = &network.identity, return); // This event must be triggered after connection
    let side_a_zones = &event.0;
    let side_b_zones = &event.1;

    let (our_zones, their_zones) = match identity.side {
        oc_root::side::Side::A => (side_a_zones, side_b_zones),
        oc_root::side::Side::B => (side_b_zones, side_a_zones),
    };

    let our_zones_names: Vec<Name> = our_zones.iter().map(|z| Name::new(z.0.clone())).collect();
    let their_zones_names: Vec<Name> = their_zones.iter().map(|z| Name::new(z.0.clone())).collect();

    for (entity, name, mut visibility, handle) in spawns {
        let is_our = our_zones_names.contains(name);
        let is_their = their_zones_names.contains(name);

        let (visibility_, alpha) = if is_our {
            commands.entity(entity).insert(OwnSpawnZone);
            (Visibility::Hidden, 0.)
        } else if is_their {
            (Visibility::Visible, SPAWN_ZONE_ENEMY_OPACITY)
        } else {
            (Visibility::Visible, SPAWN_ZONE_NONE_OPACITY)
        };

        *visibility = visibility_;
        if let Some(mut material) = materials.get_mut(&handle.0) {
            material.color.set_alpha(alpha);
        }
    }
}

pub fn on_hide_spawn_zones(
    _: On<HideSpawnZones>,
    mut spawns: Query<&mut Visibility, With<SpawnZone>>,
) {
    for mut visibility in &mut spawns {
        *visibility = Visibility::Hidden;
    }
}
