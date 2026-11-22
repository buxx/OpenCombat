use bevy::prelude::*;
use oc_geo::region::WorldRegionIndex;
use oc_physics::update::bevy::Region;
use oc_root::{WcfgFrom, WcfgInto, geo::ScreenVec2, identity::Identity, y::V};
use oc_utils::{bevy::EntityMapping, let_some};

use crate::{
    ingame::{
        draw::Z_VEHICLE,
        input::vehicle::{ForgotVehicleEvent, InsertVehicleEvent},
        region::ForgottenRegion,
    },
    sprites::vehicle::VehicleSprites,
    states::GameConfig,
    world::{VisibilitiesUpdated, World},
};

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityMapping<oc_vehicle::VehicleIndex>>()
            .add_observer(on_insert_vehicle)
            .add_observer(on_forgot_vehicle)
            .add_observer(on_forgotten_region)
            .add_observer(on_visibilities_updated)
            .add_systems(Update, animate_vehicles);
    }
}

#[derive(Debug, Component, Deref)]
pub struct VehicleIndex(pub oc_vehicle::VehicleIndex);

#[derive(Debug, Component)]
pub struct VehicleTurret;

/// Own side vehicles are always visible. Opposite side ones are visible if their chassis is.
pub fn visibility(
    identity: &Identity,
    i: oc_vehicle::VehicleIndex,
    vehicle: &oc_vehicle::Vehicle,
    world: &World,
    all: bool,
) -> Visibility {
    match all || identity.side == vehicle.side || world.vehicle_visible(i) {
        true => Visibility::Visible,
        false => Visibility::Hidden,
    }
}

#[allow(clippy::too_many_arguments)]
fn on_insert_vehicle(
    insert: On<InsertVehicleEvent>,
    mut commands: Commands,
    g: Res<GameConfig>,
    sprites: Option<Res<VehicleSprites>>,
    mut state: ResMut<EntityMapping<oc_vehicle::VehicleIndex>>,
    world: Res<World>,
    network: Res<crate::network::state::State>,
    config: Res<crate::config::Config>,
) {
    let_some!(g = &g.0, return);
    let_some!(sprites = sprites, return);
    let_some!(identity = &network.identity, return);
    let (i, vehicle) = (insert.0, &insert.1);
    tracing::trace!(name="spawn-vehicle", i=?i, position=?vehicle.position);

    // Vehicle can be sent again (when region listened again)
    if let Some(entity) = state.remove(&i) {
        commands.entity(entity).despawn();
    }

    let mod_vehicle = g.mod_.vehicle(vehicle.type_);
    let position = ScreenVec2::from_(vehicle.position, &g.w);
    let tile: oc_geo::tile::TileXy = vehicle.position.into_(&g.w);
    let tile: oc_geo::tile::WorldTileIndex = tile.into_(&g.w);
    let region: WorldRegionIndex = tile.into_(&g.w);
    let chassis_rotation = vehicle.chassis_direction.bquat(V::Gui);
    let visibility = visibility(identity, i, vehicle, &world, config.see_all_enemies);

    let entity = commands
        .spawn((
            VehicleIndex(i),
            Region(region),
            visibility,
            sprites.chassis(mod_vehicle.row),
            Transform::from_xyz(position.x, position.y, Z_VEHICLE).with_rotation(chassis_rotation),
        ))
        .id();

    if let Some(turret) = vehicle.turret_direction {
        // Turret is a child: its rotation is relative to chassis one
        let rotation = chassis_rotation.inverse() * turret.bquat(V::Gui);
        commands.entity(entity).with_child((
            VehicleTurret,
            sprites.turret(mod_vehicle.row),
            Transform::from_xyz(0., 0., 0.01).with_rotation(rotation),
        ));
    }

    state.insert(i, entity);
}

fn on_forgot_vehicle(
    forgot: On<ForgotVehicleEvent>,
    mut commands: Commands,
    mut state: ResMut<EntityMapping<oc_vehicle::VehicleIndex>>,
) {
    if let Some(entity) = state.remove(&forgot.0) {
        tracing::trace!(name = "remove-vehicle", i=?forgot.0);
        commands.entity(entity).despawn();
    }
}

fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    mut state: ResMut<EntityMapping<oc_vehicle::VehicleIndex>>,
    query: Query<(Entity, &Region, &VehicleIndex)>,
) {
    for (entity, region_, i) in query {
        if region_.0 == region.0 {
            tracing::trace!(name = "remove-vehicle", i=?i.0);
            commands.entity(entity).despawn();
            state.remove(&i.0);
        }
    }
}

fn on_visibilities_updated(
    _: On<VisibilitiesUpdated>,
    mut query: Query<(&VehicleIndex, &mut Visibility)>,
    world: Res<World>,
    network: Res<crate::network::state::State>,
    config: Res<crate::config::Config>,
) {
    let_some!(identity = &network.identity, return);

    for (i, mut visibility) in query.iter_mut() {
        let_some!(vehicle = world.vehicle(i.0), continue);
        *visibility = self::visibility(identity, i.0, vehicle, &world, config.see_all_enemies);
    }
}

/// Animate vehicles between server updates (rotations toward targets and translation by forces).
fn animate_vehicles(
    g: Res<GameConfig>,
    time: Res<Time>,
    mut world: ResMut<World>,
    mut vehicles: Query<(&VehicleIndex, &mut Transform, Option<&Children>), Without<VehicleTurret>>,
    mut turrets: Query<&mut Transform, With<VehicleTurret>>,
) {
    let_some!(g = &g.0, return);
    let delta = time.delta_secs() * g.w.speed();

    for (i, mut transform, children) in &mut vehicles {
        let_some!(vehicle = world.vehicle(i.0), continue);
        let type_ = g.mod_.vehicle(vehicle.type_);
        let (chassis, turret) = vehicle.rotate(type_, delta);
        let position = vehicle.translate(&g.w, delta);
        if (chassis, turret, position)
            == (
                vehicle.chassis_direction,
                vehicle.turret_direction,
                vehicle.position,
            )
        {
            continue;
        }

        let_some!(vehicle = world.vehicle_mut(i.0), continue);
        vehicle.chassis_direction = chassis;
        vehicle.turret_direction = turret;
        vehicle.position = position;

        let screen = ScreenVec2::from_(position, &g.w);
        let chassis_rotation = chassis.bquat(V::Gui);
        transform.translation.x = screen.x;
        transform.translation.y = screen.y;
        transform.rotation = chassis_rotation;

        let (Some(turret), Some(children)) = (turret, children) else {
            continue;
        };
        for child in children.iter() {
            if let Ok(mut transform) = turrets.get_mut(child) {
                // Turret rotation is relative to chassis one
                transform.rotation = chassis_rotation.inverse() * turret.bquat(V::Gui);
            }
        }
    }
}
