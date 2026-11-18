use bevy::prelude::*;
use oc_geo::tile::TileXy;
use oc_geo::tile::WorldTileIndex;
use oc_individual::BodyGesture;
use oc_individual::order::Order;
use oc_individual::order::OrderType;
use oc_network::ToServer;
use oc_root::Wcfg;
use oc_root::WcfgFrom;
use oc_root::WorldConfig;
use oc_root::geo::WorldVec2;
use oc_root::physics::Meters;
use oc_root::y::V;
use oc_utils::d2::Direction;
use oc_utils::let_ok;
use oc_utils::let_some;
use oc_utils::return_if;
use oc_utils::true_;

use crate::cursor_to;
use crate::entity::individual::IndividualIndex;
use crate::ingame::behavior::DespawnPendingSquadOrders;
use crate::ingame::behavior::DirectionSquadOrder;
use crate::ingame::behavior::PositionSquadOrder;
use crate::ingame::behavior::SpawnSquadOrder;
use crate::ingame::input::left_click::LeftClick;
use crate::ingame::input::left_click::LeftClickMode;
use crate::ingame::input::left_click::SetLeftClick;
use crate::ingame::lov::DespawnLov;
use crate::ingame::lov::SpawnBeginningLov;
use crate::ingame::lov::SpawnLovProfile;
use crate::ingame::path::ComputeDisplayPaths;
use crate::ingame::path::SpawnPathProfile;
use crate::ingame::path::SpawnPathProfileKey;
use crate::network::output::ToServerEvent;
use crate::states::GameConfig;
use crate::utils::drag::Phantom;
use crate::utils::hover::Hovered;

/// Marker which indicate order is pending (player prepare to give it)
#[derive(Debug, Clone, Component)]
pub struct PendingOrder;

#[allow(clippy::too_many_arguments)]
pub fn system(
    mut commands: Commands,
    w: Res<Wcfg>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    keys: Res<ButtonInput<KeyCode>>,
    world: Res<crate::world::World>,
    mut ingame: ResMut<crate::ingame::state::State>,
    drag: Query<&Phantom>,
    position_markers: Query<&PositionSquadOrder>,
    mut direction_markers: Query<(&DirectionSquadOrder, &mut Transform), With<PendingOrder>>,
) {
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let point = cursor_to!(cursor, camera, w, WorldVec2);

    let LeftClickMode::Order(order) = &mode.0 else {
        return;
    };
    tracing::trace!(name="ingame-input-left-click-order", order=?order);

    return_if!(maybe_cancel(&mut commands, &buttons, &keys, &mut ingame));
    // TODO: maybe too CPU consumer (compute it not at each frames ?)
    show(
        w,
        point,
        order,
        &mut commands,
        &mode,
        &ingame,
        &world,
        &drag,
        &position_markers,
        &mut direction_markers,
    );
}

#[allow(clippy::too_many_arguments)]
fn show(
    w: &WorldConfig,
    point: WorldVec2,
    order: &OrderType,
    commands: &mut Commands,
    mode: &LeftClick,
    ingame: &crate::ingame::state::State,
    world: &crate::world::World,
    drag: &Query<&Phantom>,
    position_markers: &Query<&PositionSquadOrder>,
    direction_markers: &mut Query<(&DirectionSquadOrder, &mut Transform), With<PendingOrder>>,
) {
    tracing::trace!(name="ingame-input-left_click-show-order", mode=?mode.0, point=?point);

    match order {
        OrderType::Idle | OrderType::Defend | OrderType::Hide => {
            rotate_direction_markers(point, world, direction_markers);
        }
        OrderType::MoveTo | OrderType::MoveFastTo | OrderType::SneakTo => {
            let spawns = path_profiles(w, point, mode, ingame, world, drag, position_markers);
            commands.trigger(ComputeDisplayPaths(spawns));
        }
        OrderType::Engage | OrderType::Suppress => {
            let spawns = lov_profiles(ingame, world, drag, position_markers, None);
            for spawn in spawns {
                commands.trigger(SpawnBeginningLov(spawn));
            }
        }
        OrderType::Grenade | OrderType::Smoke => {
            let max = Some(oc_individual::behavior::THROW_MAX_DISTANCE);
            let spawns = lov_profiles(ingame, world, drag, position_markers, max);
            for spawn in spawns {
                commands.trigger(SpawnBeginningLov(spawn));
            }
        }
    }
}

fn rotate_direction_markers(
    point: WorldVec2,
    world: &crate::world::World,
    direction_markers: &mut Query<(&DirectionSquadOrder, &mut Transform), With<PendingOrder>>,
) {
    for (order, mut transform) in direction_markers.iter_mut() {
        let_some!(squad = world.squad(order.squad()), continue);
        let reference = squad.position;
        let direction = Direction::from_points2d(reference.into(), point.into());
        *transform = transform.with_rotation(direction.bquat(V::Gui));
    }
}

fn lov_profiles(
    state: &crate::ingame::state::State,
    world: &crate::world::World,
    drag: &Query<&Phantom>,
    markers: &Query<&PositionSquadOrder>,
    max_distance: Option<Meters>,
) -> Vec<SpawnLovProfile> {
    let mut profiles = vec![];

    // Spawns points from selected squads (not when dragging a marker, which has its own)
    for squad in state.selected_squads().iter().filter(|_| drag.is_empty()) {
        tracing::trace!(name = "ingame-input-left-click-lov-selected-squads-squad", squad=?squad);
        let_some!(squad = world.squad(*squad), continue);
        let_some!(leader = world.individual(squad.leader()), continue);
        profiles.push(SpawnLovProfile {
            start: leader.position.into(),
            start_plus_z: leader.gesture.body.weapon_z(),
            stop_plus_z: BodyGesture::Prone(Direction::NORTH).target_z(),
            max: max_distance,
        });
    }

    for dragged in drag {
        let_ok!(marker = markers.get(dragged.0), continue);
        let PositionSquadOrder(squad, _) = marker;
        let_some!(squad = world.squad(*squad), continue);
        let_some!(leader = world.individual(squad.leader()), continue);

        profiles.push(SpawnLovProfile {
            start: leader.position.into(),
            start_plus_z: leader.gesture.body.weapon_z(),
            stop_plus_z: BodyGesture::Prone(Direction::NORTH).target_z(),
            max: max_distance,
        });
    }

    profiles
}

fn path_profiles(
    w: &WorldConfig,
    point: WorldVec2,
    mode: &LeftClick,
    ingame: &crate::ingame::state::State,
    world: &crate::world::World,
    drag: &Query<&Phantom>,
    markers: &Query<&PositionSquadOrder>,
) -> Vec<SpawnPathProfile> {
    let mut spawns = vec![];
    // Spawns points from selected squads
    if drag.is_empty() {
        spawns.extend(ingame.selected_squads().iter().filter_map(|i| {
            tracing::trace!(name="ingame-input-left_click-show-order-squad", mode=?mode.0, point=?point, squad=?i);
            let pending: Vec<WorldVec2> = ingame.pending_orders().iter().filter_map(|o| o.position()).collect();
            let points = [pending, vec![point]].concat();

            let squad = world.squad(*i)?;
            let leader = world.individual(squad.leader())?;
            paths_from(w, *i, points, leader.position.into())
        }).flatten().collect::<Vec<_>>());
    }

    // Spawns points from dragged order markers
    spawns.extend(
        drag.iter()
            .filter_map(|dragged| {
                let mut profiles = vec![];

                let marker = markers.get(dragged.0).ok()?;
                let PositionSquadOrder(squad, index) = marker;
                let i = *squad;
                let squad = world.squad(*squad)?;
                let orders = &squad.orders;
                if orders.is_empty() {
                    return None;
                }

                // Dragged marker is the first marker of squad markers
                if index.0 as usize == orders.len() - 1 {
                    let leader = world.individual(squad.leader())?;
                    let paths = paths_from(w, i, vec![point], leader.position.into());
                    profiles.extend(paths.unwrap_or_default())
                // If not, there is a marker before it
                } else {
                    let index = index.0 + 1;
                    let mut orders = orders.iter().rev();
                    if let Some(order) = orders.nth(index as usize)
                        && let Some(position) = order.position()
                    {
                        let paths = paths_from(w, i, vec![point], position);
                        profiles.extend(paths.unwrap_or_default())
                    }
                }

                // There is another marker after it
                if index.0 != 0 {
                    let index = index.0 - 1;
                    let mut orders = orders.iter().rev();
                    if let Some(order) = orders.nth(index as usize)
                        && let Some(position) = order.position()
                    {
                        let paths = paths_from(w, i, vec![point], position);
                        profiles.extend(paths.unwrap_or_default())
                    }
                }

                Some(profiles)
            })
            .flatten()
            .collect::<Vec<_>>(),
    );

    spawns
}

fn paths_from(
    w: &WorldConfig,
    i: oc_individual::squad::SquadIndex,
    points: Vec<WorldVec2>,
    start: WorldVec2,
) -> Option<Vec<SpawnPathProfile>> {
    let mut start = start;
    Some(
        points
            .into_iter()
            .map(|point| {
                let end: WorldVec2 = point;
                let start_tile = TileXy::from_([start.x, start.y], w);
                let start_tile = WorldTileIndex::from_(start_tile, w);
                let key = SpawnPathProfileKey::Squad {
                    i,
                    start: start_tile,
                    end,
                };
                let profile = SpawnPathProfile { key, start, end };
                start = end;
                profile
            })
            .collect::<Vec<_>>(),
    )
}

fn maybe_cancel(
    commands: &mut Commands,
    _buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
    ingame: &mut crate::ingame::state::State,
) -> bool {
    if keys.just_pressed(KeyCode::Escape) {
        tracing::trace!(name = "ingame-input-left-click-order-cancel");
        cancel(commands, ingame);
        return true;
    }
    false
}

fn cancel(commands: &mut Commands, ingame: &mut crate::ingame::state::State) {
    commands.trigger(ComputeDisplayPaths(vec![]));
    commands.trigger(SetLeftClick(LeftClickMode::Select));
    commands.trigger(DespawnLov);
    ingame.clear_pending_orders();
}

#[allow(clippy::too_many_arguments)]
pub fn on_click(
    mut click: On<PointerClick>,
    g: Res<GameConfig>,
    mut commands: Commands,
    _buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ingame: ResMut<crate::ingame::state::State>,
    camera: Single<(&Camera, &GlobalTransform)>,
    left_click: Res<LeftClick>,
    world: Res<crate::world::World>,
    individuals: Query<&IndividualIndex>,
    network: Res<crate::network::state::State>,
) {
    let_some!(g = &g.0, return);
    let_some!(identity = &network.identity, return);
    let point = click.pointer.position;
    let point = cursor_to!(point, camera, &g.w, WorldVec2);
    let individual = individuals.get(click.event_target()).ok();
    let individual = individual.map(|i| i.0);
    let target = individual.filter(|&i| world.engageable(i, identity.side.opposite()));

    match click.button {
        PointerButton::Primary => {
            let adding = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
            let LeftClickMode::Order(order_type) = left_click.0 else {
                return;
            };
            let mut order_type = order_type;
            if target.is_none() && matches!(order_type, OrderType::Engage) {
                tracing::trace!("Switch order 'engage' to 'suppress'");
                // Transform engage order into suppress if cursor is not over an opposite individual
                order_type = OrderType::Suppress;
            }
            tracing::trace!(name = "ingame-input-left-click-order-action", order_type=?order_type);

            // Its a final order give
            if !adding {
                give_orders(
                    &mut commands,
                    &ingame,
                    &world,
                    point,
                    Some(order_type),
                    target,
                );
                cancel(&mut commands, &mut ingame);
            // User want stack order
            } else {
                if let Some(order) = match order_type {
                    OrderType::MoveTo => Some(Order::MoveTo(point)),
                    OrderType::MoveFastTo => Some(Order::MoveFastTo(point)),
                    OrderType::SneakTo => Some(Order::SneakTo(point)),
                    // These orders can't be stacked
                    OrderType::Idle
                    | OrderType::Defend
                    | OrderType::Hide
                    | OrderType::Engage
                    | OrderType::Suppress
                    | OrderType::Grenade
                    | OrderType::Smoke => None,
                } {
                    ingame.push_pending_orders(order);
                }
            }
        }
        PointerButton::Secondary => {
            give_orders(&mut commands, &ingame, &world, point, None, None);
            cancel(&mut commands, &mut ingame);
        }
        PointerButton::Middle => {
            ingame.pop_pending_order();
        }
    };

    click.propagate(false);
}

fn give_orders(
    commands: &mut Commands,
    ingame: &crate::ingame::state::State,
    world: &crate::world::World,
    point: WorldVec2,
    order_type: Option<OrderType>,
    individual: Option<oc_individual::IndividualIndex>,
) {
    for squad in ingame.selected_squads() {
        // Squad without leader able to lead can't receive orders
        let_some!(squad_ = world.squad(*squad), continue);
        let_some!(leader = world.individual(squad_.leader()), continue);
        true_!(leader.can_leadership(), continue);

        let mut orders = ingame.pending_orders().to_vec();
        if let Some(order_type) = &order_type {
            let_some!(squad_ = world.squad(*squad), continue);

            // FIXME: With multiple squad, need decal a little (distance from each others ?)
            let order = match order_type {
                OrderType::Idle => Order::Idle,
                OrderType::MoveTo => Order::MoveTo(point),
                OrderType::MoveFastTo => Order::MoveFastTo(point),
                OrderType::SneakTo => Order::SneakTo(point),
                OrderType::Suppress => Order::Suppress(point),
                OrderType::Grenade => Order::Grenade(point),
                OrderType::Smoke => Order::Smoke(point),
                OrderType::Defend => {
                    let direction = Direction::from_points2d(squad_.position.into(), point.into());
                    Order::Defend(direction)
                }
                OrderType::Hide => {
                    let direction = Direction::from_points2d(squad_.position.into(), point.into());
                    Order::Hide(direction)
                }
                OrderType::Engage => match individual {
                    Some(individual) => Order::Engage(individual),
                    None => {
                        tracing::trace!(
                            "Skip giving order type 'engage' as individual not provided"
                        );
                        return;
                    }
                },
            };

            orders.push(order.clone());
        }

        let set_orders = oc_network::SquadMessage::SetOrders(orders.clone());
        commands.trigger(ToServerEvent(ToServer::Squad(*squad, set_orders)));
    }
}

/// React to left click mode change to defend/hide order and spawn position order marker on selected squads
pub fn on_set_left_click(
    event: On<SetLeftClick>,
    mut commands: Commands,
    state: Res<crate::ingame::state::State>,
) {
    // Previous preview markers (cancelled or previous order type) must not remain
    commands.trigger(DespawnPendingSquadOrders);

    if let Some(order) = event.0.order()
        && let Some(order) = match order {
            OrderType::Defend => Some(Order::Defend(Direction::NORTH)),
            OrderType::Hide => Some(Order::Hide(Direction::NORTH)),
            OrderType::Idle
            | OrderType::MoveTo
            | OrderType::MoveFastTo
            | OrderType::SneakTo
            | OrderType::Engage
            | OrderType::Suppress
            | OrderType::Grenade
            | OrderType::Smoke => None,
        }
    {
        for squad in state.selected_squads() {
            tracing::trace!(name="ingame-input-left-click-order-on-set-left-click-trigger", squad=?squad, order=?order);
            commands.trigger(SpawnSquadOrder::Direction(*squad, order.clone(), true));
        }
    }
}

/// Sprite following cursor when about to give an engage/suppress/grenade/smoke order
#[derive(Debug, Component)]
pub struct OrderCursor;

/// Draw under cursor the sprite of order which would be given: engage if there is an
/// individual under it, else suppress.
#[allow(clippy::too_many_arguments)]
pub fn cursor_sprite(
    mut commands: Commands,
    g: Res<GameConfig>,
    asset_server: Res<AssetServer>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mode: Res<LeftClick>,
    dragging: Res<crate::utils::drag::Cursor>,
    world: Res<crate::world::World>,
    network: Res<crate::network::state::State>,
    individuals: Query<(&IndividualIndex, &Hovered)>,
    mut cursors: Query<(Entity, &mut Sprite, &mut Transform), With<OrderCursor>>,
) {
    use crate::sprites::{SpriteRect, order::SquadOrderSprite};
    use oc_root::y::Y;

    // When dragging an existing marker, its phantom is the only sprite to show
    let LeftClickMode::Order(
        order @ (OrderType::Engage | OrderType::Suppress | OrderType::Grenade | OrderType::Smoke),
    ) = &mode.0
    else {
        for (entity, _, _) in &cursors {
            commands.entity(entity).despawn();
        }
        return;
    };
    if dragging.0.is_some() {
        for (entity, _, _) in &cursors {
            commands.entity(entity).despawn();
        }
        return;
    }
    let_some!(g = &g.0, return);
    let_some!(identity = &network.identity, return);
    let_some!(cursor = window.cursor_position(), return);
    let point = cursor_to!(cursor, camera, &g.w, WorldVec2);

    let opposite_hovered = individuals
        .iter()
        .any(|(i, hovered)| **hovered && world.engageable(i.0, identity.side.opposite()));
    let rect = match order {
        OrderType::Grenade => SquadOrderSprite::Grenade,
        OrderType::Smoke => SquadOrderSprite::Smoke,
        _ if opposite_hovered => SquadOrderSprite::Engage,
        _ => SquadOrderSprite::Suppress,
    }
    .rect();
    let translation = Vec3::new(
        point.x,
        point.y.to_gui_y(&g.w),
        crate::ingame::draw::Z_SQUAD_ORDER,
    );

    match cursors.single_mut() {
        // When sprite already exist, update its rect (to adapt hover target or not)
        Ok((_, mut sprite, mut transform)) => {
            sprite.rect = Some(rect);
            transform.translation = translation;
        }
        // Else, create it
        Err(_) => {
            commands.spawn((
                OrderCursor,
                Sprite {
                    image: asset_server.load(crate::ingame::draw::UI_FILE),
                    rect: Some(rect),
                    ..default()
                },
                Transform::from_translation(translation),
                // Must not capture pointer: clicks go to what is under it
                Pickable::IGNORE,
            ));
        }
    }
}

/// Cursor sprite must not remain when leaving order mode
pub fn despawn_cursor_sprite(mut commands: Commands, cursors: Query<Entity, With<OrderCursor>>) {
    for entity in &cursors {
        commands.entity(entity).despawn();
    }
}
