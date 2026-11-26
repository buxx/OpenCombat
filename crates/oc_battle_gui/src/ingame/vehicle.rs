use bevy::prelude::*;
use oc_geo::region::WorldRegionIndex;
use oc_network::{SquadMessage, ToServer};
use oc_physics::update::bevy::Region;
use oc_root::{
    WcfgFrom, WcfgInto,
    geo::{ScreenVec2, WorldVec2},
    identity::Identity,
    y::V,
};
use oc_utils::{bevy::EntityMapping, let_ok, let_some, polygon::Polygon};
use oc_world::spawn::SpawnZone;

use crate::{
    ingame::{
        BattlePhase, InGameState,
        draw::Z_VEHICLE,
        input::{
            left_click::{LeftClickMode, LeftClickModeType, SetLeftClick},
            vehicle::{ForgotVehicleEvent, InsertVehicleEvent, UpdateVehicleEvent},
        },
        region::ForgottenRegion,
        spawn::OwnSpawnZone,
    },
    network::output::ToServerEvent,
    sprites::vehicle::VehicleSprites,
    states::{AppState, GameConfig, PointerIn},
    utils::{
        drag::{self, DragPlugin, Dragged, Dragging, Phantom},
        pick_radius::PickRadius,
        selected::Selected,
    },
    world::{VisibilitiesUpdated, World},
};

/// Alpha of the vehicle phantom displayed while dragging it
const PHANTOM_ALPHA: f32 = 0.6;

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityMapping<oc_vehicle::VehicleIndex>>()
            .add_observer(on_insert_vehicle)
            .add_observer(on_update_vehicle)
            .add_observer(on_forgot_vehicle)
            .add_observer(on_forgotten_region)
            .add_observer(on_visibilities_updated)
            .add_observer(on_spawn_vehicle_phantom)
            .add_observer(on_drop_vehicle_phantom)
            .add_plugins(DragPlugin::<VehicleIndex, BattlePhase>::new(
                BattlePhase::Deployment,
            ))
            .add_systems(Update, (animate_vehicles, sync_selected));
    }
}

#[derive(Debug, Component, Deref)]
pub struct VehicleIndex(pub oc_vehicle::VehicleIndex);

#[derive(Debug, Component)]
pub struct VehicleTurret;

/// Vehicle child sprite following the turret rotation (turret and its shadow)
#[derive(Debug, Component)]
pub struct TurretPart;

/// Vehicle child sprite drawn as a shadow, with given z relative to chassis
#[derive(Debug, Component)]
pub struct Shadow(f32);

pub const DESTROYED_COLOR: Color = Color::srgb(0.45, 0.45, 0.45);
const SHADOW_OFFSET: Vec2 = Vec2::new(3., -3.);
/// Z of shadows relative to the chassis
const Z_CHASSIS_SHADOW: f32 = -0.005;
const Z_TURRET_SHADOW: f32 = 0.005;
const Z_TURRET: f32 = 0.01;

pub fn shadow_translation(chassis_rotation: Quat, z: f32) -> Vec3 {
    let offset = chassis_rotation.inverse() * SHADOW_OFFSET.extend(0.);
    Vec3::new(offset.x, offset.y, z)
}

pub fn chassis_shadow(sprites: &VehicleSprites, row: u8, chassis_rotation: Quat) -> impl Bundle {
    (
        Shadow(Z_CHASSIS_SHADOW),
        sprites.chassis_shadow(row),
        Transform::from_translation(shadow_translation(chassis_rotation, Z_CHASSIS_SHADOW)),
    )
}

/// Chassis sprite of `vehicle`: darkened when destroyed
fn chassis_sprite(sprites: &VehicleSprites, row: u8, vehicle: &oc_vehicle::Vehicle) -> Sprite {
    let mut sprite = sprites.chassis(row);
    if vehicle.destroyed {
        sprite.color = DESTROYED_COLOR;
    }
    sprite
}

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
            chassis_sprite(&sprites, mod_vehicle.row, vehicle),
            Transform::from_xyz(position.x, position.y, Z_VEHICLE).with_rotation(chassis_rotation),
        ))
        .with_child(chassis_shadow(&sprites, mod_vehicle.row, chassis_rotation))
        .id();

    // All vehicles are pickable: opposite side ones to be targeted by fire orders
    let (width, height) = mod_vehicle.size;
    let radius = (width.pixels(&g.w) + height.pixels(&g.w)) / 4.;
    commands
        .entity(entity)
        .insert((Pickable::default(), PickRadius(radius)));

    // Own side vehicles can be clicked (to select their squad) and dragged during deployment
    if identity.side == vehicle.side {
        commands
            .entity(entity)
            .insert((
                Selected::default(),
                Dragged::<VehicleIndex>::default(),
            ))
            .observe(
                on_click
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Select))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .observe(
                drag::on_drag_start::<VehicleIndex>
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(BattlePhase::Deployment))
                    .run_if(in_state(LeftClickModeType::Select))
                    .run_if(in_state(PointerIn::Battle)),
            );
    }

    // Destroyed vehicle turret is not displayed
    if let (Some(turret), false) = (vehicle.turret_direction, vehicle.destroyed) {
        // Turret is a child: its rotation is relative to chassis one
        let rotation = chassis_rotation.inverse() * turret.bquat(V::Gui);
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                TurretPart,
                Shadow(Z_TURRET_SHADOW),
                sprites.turret_shadow(mod_vehicle.row),
                Transform::from_translation(shadow_translation(chassis_rotation, Z_TURRET_SHADOW))
                    .with_rotation(rotation),
            ));
            parent.spawn((
                VehicleTurret,
                TurretPart,
                sprites.turret(mod_vehicle.row),
                Transform::from_xyz(0., 0., Z_TURRET).with_rotation(rotation),
            ));
        });
    }

    state.insert(i, entity);
}

/// Vehicle placed (ex. deployment): move it. Destroyed vehicle: remove its turret and darken its
/// chassis.
fn on_update_vehicle(
    update: On<UpdateVehicleEvent>,
    mut commands: Commands,
    g: Res<GameConfig>,
    state: Res<EntityMapping<oc_vehicle::VehicleIndex>>,
    mut chassis: Query<
        (&mut Sprite, &mut Transform, &mut Region, Option<&Children>),
        With<VehicleIndex>,
    >,
    turrets: Query<(), With<TurretPart>>,
) {
    let_some!(entity = state.get(&update.0), return);
    let_some!(
        (mut sprite, mut transform, mut region, children) = chassis.get_mut(*entity).ok(),
        return
    );

    if let oc_vehicle::Update::SetPosition(position) = update.1 {
        let_some!(g = &g.0, return);
        let screen = ScreenVec2::from_(position, &g.w);
        transform.translation.x = screen.x;
        transform.translation.y = screen.y;
        let tile: oc_geo::tile::TileXy = position.into_(&g.w);
        let tile: oc_geo::tile::WorldTileIndex = tile.into_(&g.w);
        region.0 = tile.into_(&g.w);
        return;
    }

    let oc_vehicle::Update::SetDestroyed(true) = update.1 else {
        return;
    };

    sprite.color = DESTROYED_COLOR;
    for child in children.into_iter().flat_map(|c| c.iter()) {
        if turrets.contains(child) {
            commands.entity(child).despawn();
        }
    }
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

type VehicleChildren = (
    &'static mut Transform,
    Has<TurretPart>,
    Option<&'static Shadow>,
);

/// Animate vehicles between server updates (rotations toward targets and translation by forces).
fn animate_vehicles(
    g: Res<GameConfig>,
    time: Res<Time>,
    mut world: ResMut<World>,
    mut vehicles: Query<(&VehicleIndex, &mut Transform, Option<&Children>)>,
    mut children_: Query<VehicleChildren, Without<VehicleIndex>>,
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

        let_some!(children = children, continue);
        // Turret rotation is relative to chassis one
        let turret_rotation = turret.map(|t| chassis_rotation.inverse() * t.bquat(V::Gui));
        for child in children.iter() {
            let_ok!(
                (mut transform, turret_part, shadow) = children_.get_mut(child),
                continue
            );
            if let Some(shadow) = shadow {
                transform.translation = shadow_translation(chassis_rotation, shadow.0);
            }
            if let (Some(rotation), true) = (turret_rotation, turret_part) {
                transform.rotation = rotation;
            }
        }
    }
}

/// Click on vehicle: act as a click on its crew (select its squad, open squad contextual menu)
#[allow(clippy::too_many_arguments)]
fn on_click(
    mut event: On<PointerClick>,
    mut commands: Commands,
    mut state: ResMut<super::state::State>,
    world: Res<World>,
    vehicles: Query<&VehicleIndex>,
    mut selected: Query<&mut Selected>,
) {
    let_ok!(
        vehicle = vehicles.get(event.original_event_target()),
        return
    );
    let_some!(
        member = world.vehicle_crew(vehicle.0).first().copied(),
        return
    );
    super::individual::click(
        event.button,
        member,
        &mut commands,
        &mut state,
        &world,
        &mut selected,
    );
    event.propagate(false);
}

/// Vehicles are selected when their crew is
fn sync_selected(
    world: Res<World>,
    individuals: Res<EntityMapping<oc_individual::IndividualIndex>>,
    crew: Query<&Selected, Without<VehicleIndex>>,
    mut vehicles: Query<(&VehicleIndex, &mut Selected)>,
) {
    for (vehicle, mut selected) in &mut vehicles {
        let selected_ = world.vehicle_crew(vehicle.0).iter().any(|i| {
            individuals
                .get(i)
                .and_then(|entity| crew.get(*entity).ok())
                .is_some_and(|s| s.0)
        });
        if selected.0 != selected_ {
            selected.0 = selected_;
        }
    }
}

#[derive(Debug, Event)]
pub struct SpawnVehiclePhantom(Phantom);

#[derive(Debug, Event)]
pub struct DropVehiclePhantom(Entity, WorldVec2);

impl Dragging for VehicleIndex {
    fn spawn(commands: &mut Commands, marker: Phantom) {
        commands.trigger(SpawnVehiclePhantom(marker));
    }

    fn drop(commands: &mut Commands, subject: Entity, _point: WorldVec2, offset: WorldVec2) {
        commands.trigger(DropVehiclePhantom(subject, offset));
    }

    fn visual() -> drag::Visual {
        drag::Visual::Offset
    }
}

fn on_spawn_vehicle_phantom(
    event: On<SpawnVehiclePhantom>,
    mut commands: Commands,
    g: Res<GameConfig>,
    sprites: Option<Res<VehicleSprites>>,
    world: Res<World>,
    vehicles: Query<(&VehicleIndex, &Transform)>,
) {
    let_some!(g = &g.0, return);
    let_some!(sprites = sprites, return);
    let phantom = event.0;
    let_ok!((i, transform) = vehicles.get(phantom.0), return);
    let_some!(vehicle = world.vehicle(i.0), return);

    let mut sprite = sprites.chassis(g.mod_.vehicle(vehicle.type_).row);
    sprite.color = Color::srgba(1., 1., 1., PHANTOM_ALPHA);

    tracing::trace!(name="ingame-vehicle-on-spawn-vehicle-phantom", i=?i.0);
    commands.spawn((
        sprite,
        Transform::from_translation(transform.translation + Vec3::Z * 0.1)
            .with_rotation(transform.rotation),
        phantom,
    ));
    commands.trigger(SetLeftClick(LeftClickMode::Dragging));
}

/// Move vehicle squad leader (embarked in the vehicle) by the drag offset
fn on_drop_vehicle_phantom(
    event: On<DropVehiclePhantom>,
    mut commands: Commands,
    world: Res<World>,
    vehicles: Query<&VehicleIndex>,
    zones: Query<&SpawnZone, With<OwnSpawnZone>>,
) {
    let (subject, offset) = (event.0, event.1);
    commands.trigger(SetLeftClick(LeftClickMode::Select));
    let_ok!(i = vehicles.get(subject), return);
    let_some!(vehicle = world.vehicle(i.0), return);
    let_some!(member = world.vehicle_crew(i.0).first().copied(), return);
    let_some!((squad, _) = world.individual_squad(member), return);

    let drop = WorldVec2::new(vehicle.position.x + offset.x, vehicle.position.y + offset.y);
    if !zones.iter().any(|zone| zone.contains(drop)) {
        tracing::trace!(name="ingame-vehicle-on-drop-vehicle-outside-own-spawn", drop=?drop);
        return;
    }

    tracing::trace!(name="ingame-vehicle-on-drop-vehicle-set-leader-position", squad=?squad, position=?drop);
    let message = SquadMessage::SetLeaderPosition(drop);
    commands.trigger(ToServerEvent(ToServer::Squad(squad, message)));
}
