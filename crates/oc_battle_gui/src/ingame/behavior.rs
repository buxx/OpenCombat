use bevy::prelude::*;
use oc_individual::{
    order::{Order, OrderIndex, OrderType},
    squad::SquadIndex,
};
use oc_network::{SquadMessage, ToServer};
use oc_physics::update::bevy::SetPositionEvent;
use oc_root::{
    geo::WorldVec2,
    y::{V, Y},
};
use oc_utils::{collections::InvertedIndex, d2::Direction, let_ok, let_some};
use rustc_hash::FxHashMap;

use crate::{
    entity::individual::IndividualIndex,
    ingame::{
        BattlePhase, InGameState,
        draw::{self, UI_FILE, Z_SQUAD_ORDER},
        input::left_click::{LeftClickMode, LeftClickModeType, SetLeftClick, order::PendingOrder},
        lov::DespawnLov,
        path::ComputeDisplayPaths,
        region::{ForgottenRegion, ListeningRegion},
    },
    network::output::ToServerEvent,
    sprites::{IntoIndividualSprite, IntoSprite, SpriteRect, order::SquadOrderSprite},
    states::{AppState, GameConfig},
    utils::{
        drag::{self, DragPlugin, Dragged, Dragging, Phantom},
        selected::Selected,
    },
    world::World,
};

pub struct BehaviorPlugin;

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct IndividualOrders(FxHashMap<oc_individual::IndividualIndex, Vec<(Order, Entity)>>);

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct SquadOrders(FxHashMap<oc_individual::squad::SquadIndex, Vec<(Order, Entity)>>);

#[derive(Debug, Event)]
pub struct RefreshIndividualOrdersEvent(
    pub oc_individual::IndividualIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event, PartialEq)]
pub struct RefreshSquadsOrdersEvent(
    pub oc_individual::squad::SquadIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event)]
pub struct SpawnIndividualOrder(oc_individual::IndividualIndex, oc_individual::order::Order);

#[derive(Debug, Event)]
pub enum SpawnSquadOrder {
    Position(
        oc_individual::squad::SquadIndex,
        oc_individual::order::OrderIndex,
        oc_individual::order::Order,
    ),
    Direction(
        oc_individual::squad::SquadIndex,
        oc_individual::order::Order,
        bool, // Pending (true when player is giving order)
    ),
}

impl SpawnSquadOrder {
    pub fn squad(&self) -> SquadIndex {
        match self {
            SpawnSquadOrder::Position(squad, _, _) | SpawnSquadOrder::Direction(squad, _, _) => {
                *squad
            }
        }
    }

    pub fn order(&self) -> &Order {
        match self {
            SpawnSquadOrder::Position(_, _, order) | SpawnSquadOrder::Direction(_, order, _) => {
                order
            }
        }
    }
}

#[derive(Debug, Event)]
pub struct SpawnSquadOrders(
    pub oc_individual::squad::SquadIndex,
    pub Vec<oc_individual::order::Order>,
);

#[derive(Debug, Event)]
pub struct DespawnIndividualOrder(
    pub oc_individual::IndividualIndex,
    pub oc_individual::order::Order,
);

#[derive(Debug, Event)]
pub struct DespawnIndividualOrders(pub oc_individual::IndividualIndex);

#[derive(Debug, Event)]
pub struct DespawnSquadOrders(pub oc_individual::squad::SquadIndex);

#[derive(Debug, Event)]
pub struct DespawnPendingSquadOrders;

#[derive(Debug, Event)]
pub struct DespawnSquadOrder(
    pub oc_individual::squad::SquadIndex,
    oc_individual::order::Order,
);

#[derive(Debug, Component)]
pub struct PositionSquadOrder(pub SquadIndex, pub OrderIndex);

impl Dragging for PositionSquadOrder {
    fn spawn(commands: &mut Commands, marker: Phantom) {
        commands.trigger(SpawnSquadOrderMarkerPhantom(marker));
    }

    fn drop(commands: &mut Commands, subject: Entity, point: WorldVec2, _offset: WorldVec2) {
        commands.trigger(DropSquadOrderMarkerPhantom(subject, point));
    }

    fn visual() -> drag::Visual {
        drag::Visual::Offset
    }
}

#[derive(Debug, Event)]
pub struct SpawnSquadOrderMarkerPhantom(Phantom);

#[derive(Debug, Event)]
pub struct DropSquadOrderMarkerPhantom(Entity, WorldVec2);

#[derive(Debug, Clone, Component)]
pub enum DirectionSquadOrder {
    Defend(SquadIndex),
    Hide(SquadIndex),
}

#[derive(Debug, Event)]
pub struct EnterDragDirectionSquadOrderMarker(Phantom);

#[derive(Debug, Event)]
pub struct UpdateDirectionSquadOrderTarget(Entity, WorldVec2);

impl Dragging for DirectionSquadOrder {
    fn spawn(commands: &mut Commands, marker: Phantom) {
        commands.trigger(EnterDragDirectionSquadOrderMarker(marker));
    }

    fn drop(commands: &mut Commands, subject: Entity, point: WorldVec2, _offset: WorldVec2) {
        commands.entity(subject).remove::<Phantom>();
        tracing::trace!(name = "ingame-behavior-dragging-direction-squad-order-drop-trigger",);
        commands.trigger(UpdateDirectionSquadOrderTarget(subject, point));
    }

    fn visual() -> drag::Visual {
        drag::Visual::Direction
    }
}

impl DirectionSquadOrder {
    pub fn squad(&self) -> SquadIndex {
        match self {
            DirectionSquadOrder::Defend(squad) | DirectionSquadOrder::Hide(squad) => *squad,
        }
    }
}

pub fn on_spawn_squad_order_marker_phantom(
    event: On<SpawnSquadOrderMarkerPhantom>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    query: Query<&mut PositionSquadOrder>,
    world: Res<crate::world::World>,
) {
    let_ok!(order = query.get(event.0.0), return);
    let (squad, order) = (order.0, order.1);
    let_some!(squad = world.squad(squad), return);
    let_some!(order = squad.orders.get_r(order.0 as usize), return);
    let image = asset_server.load(UI_FILE);
    let rect = order.sprite().rect();
    let marker = event.0;
    let sprite = Sprite {
        image,
        rect: Some(rect),
        ..default()
    };
    commands.spawn((sprite, Transform::default(), marker));
    commands.trigger(SetLeftClick(LeftClickMode::Order(order.order_type())));
}

pub fn on_drop_squad_order_marker_phantom(
    event: On<DropSquadOrderMarkerPhantom>,
    mut commands: Commands,
    query: Query<&mut PositionSquadOrder>,
    world: Res<crate::world::World>,
    network: Res<crate::network::state::State>,
) {
    let_ok!(order = query.get(event.0), return);
    let (squad, index, position) = (order.0, order.1, event.1);
    // If
    let message = match engage_orders(squad, index, position, &world, &network) {
        Some(orders) => SquadMessage::SetOrders(orders),
        None => SquadMessage::SetPositionOrderPosition(index, position),
    };
    tracing::trace!(name = "ingame-behavior-on-drop-squad-order-marker-phantom", squad=?squad, index=?index, position=?position);
    commands.trigger(ToServerEvent(ToServer::Squad(squad, message)));
    commands.trigger(SetLeftClick(LeftClickMode::Select));
    commands.trigger(ComputeDisplayPaths(vec![]));
    commands.trigger(DespawnLov);
}

/// World pixels around a dragged marker where an individual is considered as under it
const DROP_RADIUS: f32 = 10.;

/// When dragged order is an engage or suppress. It is an engage order if dropped on an individual
/// else a suppress.
fn engage_orders(
    squad: SquadIndex,
    index: OrderIndex,
    point: WorldVec2,
    world: &crate::world::World,
    network: &crate::network::state::State,
) -> Option<Vec<Order>> {
    let identity = network.identity.as_ref()?;
    let squad_ = world.squad(squad)?;
    let mut orders = squad_.orders.clone();
    let i = orders.len().checked_sub(1 + index.0 as usize)?;
    if !matches!(orders[i], Order::Engage(_) | Order::Suppress(_)) {
        return None;
    }

    let target = world.individual_nearest(point, DROP_RADIUS, identity.side.opposite());
    tracing::trace!(name = "ingame-behavior-engage-orders-drop", point=?point, target=?target);

    // If order dropped on individual, engage, else, suppress
    orders[i] = match target {
        Some(target) => Order::Engage(target),
        None => Order::Suppress(point),
    };

    Some(orders)
}

pub fn on_enter_drag_direction_squad_order_marker(
    event: On<EnterDragDirectionSquadOrderMarker>,
    mut commands: Commands,
    markers: Query<&DirectionSquadOrder>,
) {
    let phantom = event.0;
    let_ok!(order = markers.get(phantom.0), return);
    let order_type = match order {
        DirectionSquadOrder::Defend(_) => OrderType::Defend,
        DirectionSquadOrder::Hide(_) => OrderType::Hide,
    };

    tracing::trace!(name = "ingame-behavior-on-enter-drag-direction-squad-order-marker");
    // The position order become the phantom itself
    commands.entity(phantom.0).insert(phantom);
    // Prevent other system/observer
    commands.trigger(SetLeftClick(LeftClickMode::Order(order_type)));
}

pub fn on_update_direction_squad_order_target(
    event: On<UpdateDirectionSquadOrderTarget>,
    mut commands: Commands,
    query: Query<&mut DirectionSquadOrder>,
    world: Res<crate::world::World>,
) {
    let_ok!(order = query.get(event.0), return);
    let (squad, target) = (order.squad(), event.1);
    let_some!(squad_ = world.squad(order.squad()), return);
    let reference = squad_.position;
    let direction = Direction::from_points2d(reference.into(), target.into());

    let order = match order {
        DirectionSquadOrder::Defend(_) => Order::Defend(direction),
        DirectionSquadOrder::Hide(_) => Order::Hide(direction),
    };

    tracing::trace!(name="ingame-behavior-on-update-direction-squad-order-target", squad=?squad, order=?order);
    let orders = SquadMessage::SetOrders(vec![order]);
    commands.trigger(ToServerEvent(ToServer::Squad(squad, orders)));
    commands.trigger(SetLeftClick(LeftClickMode::Select));
}

impl Plugin for BehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DragPlugin::<PositionSquadOrder, InGameState>::new(
            InGameState::Battle,
        ))
        .add_plugins(DragPlugin::<DirectionSquadOrder, InGameState>::new(
            InGameState::Battle,
        ))
        .add_plugins(DragPlugin::<IndividualIndex, BattlePhase>::new(
            BattlePhase::Deployment,
        ))
        .init_resource::<IndividualOrders>()
        .init_resource::<SquadOrders>()
        .add_observer(on_refresh_individual_orders)
        .add_observer(on_spawn_individual_order)
        .add_observer(on_despawn_individual_order)
        .add_observer(on_despawn_individual_orders)
        .add_observer(on_refresh_squad_orders)
        .add_observer(on_spawn_squad_order)
        .add_observer(on_spawn_squad_orders)
        .add_observer(on_despawn_squad_order)
        .add_observer(on_despawn_squad_orders)
        .add_observer(on_despawn_pending_squad_orders)
        .add_observer(on_listening_region)
        .add_observer(on_forgotten_region)
        .add_observer(on_spawn_squad_order_marker_phantom)
        .add_observer(on_drop_squad_order_marker_phantom)
        .add_observer(on_enter_drag_direction_squad_order_marker)
        .add_observer(on_set_position)
        .add_observer(on_update_direction_squad_order_target)
        .add_systems(
            Update,
            (follow_engage_targets, update_dragged_engage_sprites)
                .run_if(in_state(InGameState::Battle)),
        );
    }
}

/// Dragged engage/suppress marker shows the sprite of what it would become if dropped here
fn update_dragged_engage_sprites(
    world: Res<crate::world::World>,
    network: Res<crate::network::state::State>,
    g: Res<GameConfig>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    markers: Query<&PositionSquadOrder>,
    mut phantoms: Query<(&Phantom, &mut Sprite)>,
) {
    use oc_root::WcfgFrom;

    let_some!(g = &g.0, return);
    let_some!(identity = &network.identity, return);
    let_some!(cursor = window.cursor_position(), return);
    let point = crate::cursor_to!(cursor, camera, &g.w, WorldVec2);

    for (phantom, mut sprite) in &mut phantoms {
        let_ok!(
            PositionSquadOrder(squad, index) = markers.get(phantom.0),
            continue
        );
        let_some!(squad = world.squad(*squad), continue);
        let_some!(order = squad.orders.get_r(index.0 as usize), continue);
        if !matches!(order, Order::Engage(_) | Order::Suppress(_)) {
            continue;
        }

        let target = world.individual_nearest(point, DROP_RADIUS, identity.side.opposite());
        let rect = match target {
            Some(_) => SquadOrderSprite::Engage,
            None => SquadOrderSprite::Suppress,
        }
        .rect();
        if sprite.rect != Some(rect) {
            sprite.rect = Some(rect);
        }
    }
}

/// Engage order has no own position: its marker is placed on, and follows, its target
fn follow_engage_targets(
    g: Res<GameConfig>,
    world: Res<crate::world::World>,
    cursor: Res<drag::Cursor>,
    mut markers: Query<(&PositionSquadOrder, &mut Transform)>,
) {
    let_some!(g = &g.0, return);
    // Dragged marker must not be moved under the cursor
    if cursor.0.is_some() {
        return;
    }

    for (PositionSquadOrder(squad, index), mut transform) in &mut markers {
        let_some!(squad = world.squad(*squad), continue);
        let_some!(order = squad.orders.get_r(index.0 as usize), continue);
        let Order::Engage(target) = order else {
            continue;
        };
        let_some!(target = world.individual(*target), continue);
        let translation = Vec3::new(
            target.position.x,
            target.position.y.to_gui_y(&g.w),
            transform.translation.z,
        );
        if transform.translation != translation {
            transform.translation = translation;
        }
    }
}

fn on_refresh_individual_orders(event: On<RefreshIndividualOrdersEvent>, mut commands: Commands) {
    let (i, orders_) = (event.0, &event.1);
    tracing::trace!(name = "ingame-behavior-on-refresh-individual-orders", i=?i, orders=?orders_);

    // Despawn all then spawn all to ensure a total refresh
    commands.trigger(DespawnIndividualOrders(i));
    for order in orders_ {
        commands.trigger(SpawnIndividualOrder(i, order.clone()));
    }
}

fn on_refresh_squad_orders(event: On<RefreshSquadsOrdersEvent>, mut commands: Commands) {
    let (i, orders_) = (event.0, &event.1);
    tracing::trace!(name = "ingame-behavior-on-refresh-squad-orders", i=?i, order=?orders_);

    // Despawn all then spawn all to ensure a total refresh
    commands.trigger(DespawnSquadOrders(i));

    let spawns: Vec<_> = orders_
        .iter()
        .rev()
        .enumerate()
        .filter_map(|(o, order)| match order {
            Order::Idle => None,
            Order::MoveTo(_)
            | Order::MoveFastTo(_)
            | Order::SneakTo(_)
            | Order::Engage(_)
            | Order::Suppress(_)
            | Order::Grenade(_)
            | Order::Smoke(_) => Some(SpawnSquadOrder::Position(
                i,
                OrderIndex::from(o),
                order.clone(),
            )),
            Order::Defend(_) | Order::Hide(_) => {
                Some(SpawnSquadOrder::Direction(i, order.clone(), false))
            }
        })
        .collect();

    for spawn in spawns {
        commands.trigger(spawn);
    }
}

fn on_spawn_individual_order(
    event: On<SpawnIndividualOrder>,
    g: Res<GameConfig>,
    mut orders: ResMut<IndividualOrders>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);
    let image = asset_server.load(UI_FILE);
    let_some!(position = event.1.position(), return);
    let rect = event.1.individual_sprite().rect();
    let x = position.x;
    let y = position.y;
    let translation = Vec3::new(x, y.to_gui_y(&g.w), draw::Z_INDIV_ORDER);

    tracing::trace!(name = "ingame-behavior-on-spawn-individual-order-spawn", i=?event.0, position=?position, rect=?rect, translation=?translation);

    let sprite = Sprite {
        image,
        rect: Some(rect),
        ..default()
    };
    let transform = Transform::from_translation(translation);
    let entity = commands.spawn((sprite, transform)).id();

    orders
        .entry(event.0)
        .or_default()
        .push((event.1.clone(), entity));
}

fn on_despawn_individual_order(
    event: On<DespawnIndividualOrder>,
    mut orders: ResMut<IndividualOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavior-on-despawn-individual-order", i=?event.0, order=?event.1);
    if let Some(orders) = orders.get_mut(&event.0)
        && let Some(x) = orders.iter().position(|(o, _)| o == &event.1)
    {
        let (_, entity) = orders.remove(x);
        commands.entity(entity).despawn();
    }
}

fn on_despawn_individual_orders(
    event: On<DespawnIndividualOrders>,
    mut orders: ResMut<IndividualOrders>,
    mut commands: Commands,
) {
    if let Some(orders) = orders.get_mut(&event.0) {
        for (_, entity) in orders {
            commands.entity(*entity).despawn();
        }
    }
    orders.remove(&event.0);
}

/// Spawn squad orders in listened region
fn on_listening_region(region: On<ListeningRegion>, world: Res<World>, mut commands: Commands) {
    if let Some(squads) = world.squads.get(&region.0) {
        for (i, squad) in squads {
            commands.trigger(SpawnSquadOrders(*i, squad.orders.clone()))
        }
    }
}

/// Despawn squad orders in forgotten region
fn on_forgotten_region(region: On<ForgottenRegion>, world: Res<World>, mut commands: Commands) {
    if let Some(squads) = world.squads.get(&region.0) {
        for i in squads.keys() {
            commands.trigger(DespawnSquadOrders(*i))
        }
    }
}

fn on_spawn_squad_order(
    event: On<SpawnSquadOrder>,
    g: Res<GameConfig>,
    mut orders: ResMut<SquadOrders>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    world: Res<crate::world::World>,
) {
    let_some!(g = &g.0, return);
    let image = asset_server.load(UI_FILE);
    let (squad, order) = (event.squad(), event.order());
    let rect = order.sprite().rect();

    tracing::trace!(name = "ingame-behavior-on-spawn-squad-orders-spawn", i=?squad, event=?event);

    let sprite = Sprite {
        image,
        rect: Some(rect),
        ..default()
    };
    let entity = match *event {
        SpawnSquadOrder::Position(_, index, _) => {
            let marker = PositionSquadOrder(squad, index);

            // Engage has no own position: it is displayed on its target
            let position = match order {
                Order::Engage(target) => {
                    let_some!(target = world.individual(*target), return);
                    target.position.into()
                }
                _ => {
                    let_some!(position = order.position(), return);
                    position
                }
            };
            let x = position.x;
            let y = position.y;
            let translation = Vec3::new(x, y.to_gui_y(&g.w), draw::Z_SQUAD_ORDER);
            let transform = Transform::from_translation(translation);

            tracing::trace!(name = "ingame-behavior-on-spawn-squad-orders-spawn-position", i=?squad, order=?order);

            commands
                .spawn((
                    marker,
                    sprite,
                    transform,
                    Pickable::default(),
                    Selected::default(),
                    Dragged::<PositionSquadOrder>::default(),
                ))
                .observe(
                    drag::on_drag_start::<PositionSquadOrder>
                        .run_if(in_state(AppState::InGame))
                        .run_if(in_state(InGameState::Battle))
                        .run_if(in_state(LeftClickModeType::Select)),
                )
                .id()
        }
        SpawnSquadOrder::Direction(_, _, pending) => {
            let (marker, direction) = match order {
                Order::Idle
                | Order::MoveTo(_)
                | Order::MoveFastTo(_)
                | Order::SneakTo(_)
                | Order::Engage(_)
                | Order::Suppress(_)
                | Order::Grenade(_)
                | Order::Smoke(_) => return,
                Order::Defend(direction) => (DirectionSquadOrder::Defend(squad), direction),
                Order::Hide(direction) => (DirectionSquadOrder::Hide(squad), direction),
            };
            let_some!(squad_ = world.squad(squad), return);
            let point = squad_.position;
            let rotation = direction.bquat(V::Gui);
            let transform = Transform::from_xyz(point.x, point.y.to_gui_y(&g.w), Z_SQUAD_ORDER);
            let transform = transform.with_rotation(rotation);

            tracing::trace!(name = "ingame-behavior-on-spawn-squad-orders-spawn-direction", i=?squad, order=?order);
            let mut entity = commands.spawn((
                marker,
                sprite,
                transform,
                Pickable::default(),
                Selected::default(),
                Dragged::<DirectionSquadOrder>::default(),
            ));
            if pending {
                entity.insert(PendingOrder);
            }

            entity
                .observe(
                    drag::on_drag_start::<DirectionSquadOrder>
                        .run_if(in_state(AppState::InGame))
                        .run_if(in_state(InGameState::Battle))
                        .run_if(in_state(LeftClickModeType::Select)),
                )
                .id()
        }
    };

    orders
        .entry(squad)
        .or_default()
        .push((order.clone(), entity));
}

fn on_spawn_squad_orders(event: On<SpawnSquadOrders>, mut commands: Commands) {
    let (i, orders) = (event.0, &event.1);

    for (o, order) in orders.iter().rev().enumerate() {
        if let Some(event) = match order {
            Order::Idle => None,
            Order::MoveTo(_)
            | Order::MoveFastTo(_)
            | Order::SneakTo(_)
            | Order::Engage(_)
            | Order::Suppress(_)
            | Order::Grenade(_)
            | Order::Smoke(_) => Some(SpawnSquadOrder::Position(
                i,
                OrderIndex::from(o),
                order.clone(),
            )),
            Order::Defend(_) | Order::Hide(_) => {
                Some(SpawnSquadOrder::Direction(i, order.clone(), false))
            }
        } {
            commands.trigger(event);
        }
    }
}

fn on_despawn_squad_order(
    event: On<DespawnSquadOrder>,
    mut orders: ResMut<SquadOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavior-on-despawn-squad-order", i=?event.0, event=?event);
    if let Some(orders) = orders.get_mut(&event.0)
        && let Some(x) = orders.iter().position(|(o, _)| o == &event.1)
    {
        let (_, entity) = orders.remove(x);
        commands.entity(entity).despawn();
    }
}

fn on_despawn_pending_squad_orders(
    _event: On<DespawnPendingSquadOrders>,
    mut orders: ResMut<SquadOrders>,
    pending: Query<Entity, With<PendingOrder>>,
    mut commands: Commands,
) {
    for entity in pending.iter() {
        commands.entity(entity).despawn();
    }
    for orders in orders.values_mut() {
        orders.retain(|(_, entity)| !pending.contains(*entity));
    }
}

fn on_despawn_squad_orders(
    event: On<DespawnSquadOrders>,
    mut orders: ResMut<SquadOrders>,
    mut commands: Commands,
) {
    tracing::trace!(name = "ingame-behavior-on-despawn-squad-orders", event=?event);
    if let Some(orders) = orders.get_mut(&event.0) {
        for (_, entity) in orders {
            commands.entity(*entity).despawn();
        }
    }
    orders.remove(&event.0);
}

// Not very CPU optimal but attach DirectionSquadOrder as leader child imply inherit rotation and we don't want it
fn on_set_position(
    position: On<SetPositionEvent<oc_individual::IndividualIndex>>,
    g: Res<GameConfig>,
    mut markers: Query<(&mut Transform, &DirectionSquadOrder)>,
    world: Res<crate::world::World>,
) {
    let_some!(g = &g.0, return);

    for (mut transform, marker) in markers.iter_mut() {
        let_some!(squad = world.squad(marker.squad()), continue);
        if position.0 == squad.leader() {
            transform.translation = position.1.into();
            transform.translation = transform.translation.to_gui_y(&g.w);
        }
    }
}
