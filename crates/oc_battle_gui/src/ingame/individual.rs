use bevy::{prelude::*, sprite::Anchor};
use bevy_spritesheet_animation::prelude::*;
use oc_geo::region::WorldRegionIndex;
use oc_mod::magazine::MagazineIndex;
use oc_mod::weapons::WeaponType;
use oc_network::{SquadMessage, ToServer};
use oc_physics::Physic;
use oc_physics::collision::{Material, Material_};
use oc_physics::update::bevy::{Forces, PhysicsPlugin, Position, Region, Tile, Volumes};
use oc_root::WcfgFrom;
use oc_root::geo::WorldVec2;
use oc_root::geo::{ScreenVec2, WorldVec3};
use oc_root::identity::Identity;
use oc_root::y::V;
use oc_utils::bevy::EntityMapping;
use oc_utils::let_ok;
use oc_utils::polygon::Polygon;
use oc_utils::{let_some, true_};
use oc_world::spawn::SpawnZone;

use crate::entity::individual::{Behavior, IndividualIndex, Intent, Orders, Side};
use crate::ingame::behavior::{
    DespawnIndividualOrder, DespawnIndividualOrders, RefreshIndividualOrdersEvent,
};
use crate::ingame::draw::Z_INDIVIDUAL;
use crate::ingame::input::individual::{
    InsertIndividualEvent, UpdateIndividualEvent, UpdateIndividualPhysicsEvent,
};
use crate::ingame::input::left_click::select::Select;
use crate::ingame::input::left_click::{LeftClickMode, LeftClickModeType, SetLeftClick};
use crate::ingame::physics::Direction;
use crate::ingame::region::ForgottenRegion;
use crate::ingame::spawn::OwnSpawnZone;
use crate::ingame::squad::menu::contextual::{
    self, PrepareOpenSquadContextualMenu, on_prepare_open_squad_contextual_menu,
};
use crate::ingame::{self, BattlePhase, InGameState};
use crate::menu::contextual::close::CloseContextMenu;
use crate::network::output::ToServerEvent;
use crate::sprites::IntoAnimation;
use crate::sprites::outline::IndividualOutline;
use crate::sprites::soldier::{SoldierAnimationInfos, SoldierAnimations};
use crate::sprites::weapon::{
    FLASH_X, FLASH_Y, IndividualMuzzleFlash, IndividualWeapon, MuzzleFlashOf, WeaponAnimations,
};
use crate::states::{AppState, GameConfig, PointerIn};
use crate::utils::drag::{self, Dragged, Dragging, Phantom};
use crate::utils::hover::{self, Hovered, HoveredPlugin};
use crate::utils::pick_radius::{PickRadius, PickRadiusPlugin};
use crate::utils::selected::{self, Selected, SelectedPlugin};
use crate::world::World;

#[cfg(feature = "debug")]
#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct PositionsGizmos;

#[derive(Debug, Deref, Event)]
pub struct ForgotIndividual(pub oc_individual::IndividualIndex);

#[derive(Debug, Deref, Event)]
pub struct RefreshRender(pub oc_individual::IndividualIndex);

/// Declare one event by `oc_individual::Update` variant, and `trigger_update_event` which
/// triggers the event matching an update.
macro_rules! individual_update_events {
    (
        values { $($variant:ident($ty:ty) => $event:ident $(, refresh = $refresh:literal)?;)* }
        signals { $($signal:ident => $signal_event:ident;)* }
    ) => {
        $(
            #[derive(Debug, Clone, Event)]
            pub struct $event(pub oc_individual::IndividualIndex, pub $ty);
        )*

        $(
            #[derive(Debug, Clone, Event, Deref)]
            pub struct $signal_event(pub oc_individual::IndividualIndex);
        )*

        /// Trigger the event matching `update`. Return true if individual render must be refreshed
        fn trigger_update_event(
            commands: &mut Commands,
            i: oc_individual::IndividualIndex,
            update: &oc_individual::Update,
        ) -> bool {
            match update {
                $(
                    oc_individual::Update::$variant(value) => {
                        commands.trigger($event(i, value.clone()));
                        false $(|| $refresh)?
                    }
                )*
                $(
                    oc_individual::Update::$signal => {
                        commands.trigger($signal_event(i));
                        false
                    }
                )*
            }
        }
    };
}

individual_update_events! {
    values {
        SetPosition(WorldVec3) => SetPositionEvent;
        SetBehavior(oc_individual::behavior::Behavior) => SetBehaviorEvent;
        SetOrders(Vec<oc_individual::order::Order>) => SetOrdersEvent;
        SetForces(Vec<oc_physics::Force>) => SetForcesEvent;
        SetStatus(oc_individual::Status) => SetStatusEvent, refresh = true;
        SetGesture(oc_individual::Gesture) => SetGestureEvent, refresh = true;
        SetIntent(oc_individual::behavior::Intent) => SetIntentEvent;
        SetWeapons(oc_individual::Weapons) => SetWeaponsEvent;
        SetSuppress(oc_root::behavior::Suppress) => SetSuppressEvent;
        SetMagazines(Vec<MagazineIndex>) => SetMagazinesEvent;
    }
    signals {
        Accomplished => AccomplishedEvent;
        MoveStepAccomplished => MoveStepAccomplishedEvent;
    }
}

/// Observer which copies event value into individual component and into world individual field
macro_rules! sync_individual_field {
    ($name:ident, $event:ty, $component:ty, $field:ident) => {
        fn $name(
            event: On<$event>,
            mut query: Query<&mut $component>,
            state: Res<EntityMapping<oc_individual::IndividualIndex>>,
            mut world: ResMut<crate::world::World>,
        ) {
            let_some!(entity = state.get(&event.0), return);
            let_ok!(mut component = query.get_mut(*entity), return);
            let_some!(individual = world.get_individual_mut(event.0), return);
            tracing::trace!(
                name = concat!("update-individual-set-", stringify!($field)),
                i = ?event.0,
                value = ?event.1
            );

            component.0 = event.1.clone();
            individual.$field = event.1.clone();
        }
    };
}

sync_individual_field!(on_set_intent_event, SetIntentEvent, Intent, intent);
sync_individual_field!(on_set_behavior_event, SetBehaviorEvent, Behavior, behavior);
sync_individual_field!(on_set_forces_event, SetForcesEvent, Forces, forces);
sync_individual_field!(on_set_weapons_event, SetWeaponsEvent, Weapons, weapons);
sync_individual_field!(on_set_suppress_event, SetSuppressEvent, Suppress, suppress);
sync_individual_field!(
    on_set_magazines_event,
    SetMagazinesEvent,
    Magazines,
    magazines
);

#[derive(Debug, Deref, Component)]
pub struct Status(pub oc_individual::Status);

#[derive(Debug, Deref, Component)]
pub struct Weapons(pub oc_individual::Weapons);

#[derive(Debug, Deref, Component)]
pub struct Gesture(pub oc_individual::Gesture);

#[derive(Debug, Deref, Component)]
pub struct Suppress(pub oc_root::behavior::Suppress);

#[derive(Debug, Deref, Component)]
pub struct Magazines(pub Vec<MagazineIndex>);

#[cfg(feature = "debug")]
pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-individual-setup");
    let (gizmos, _) = config.config_mut::<PositionsGizmos>();
    gizmos.line.width = 2.0;
}

#[cfg(feature = "debug")]
fn positions(
    g: Res<GameConfig>,
    individuals: Query<(&IndividualIndex, &Position, &Behavior)>,
    mut gizmos: Gizmos<PositionsGizmos>,
    world: Res<World>,
) {
    let_some!(g = &g.0, return);

    for (i, position, behavior) in individuals {
        let_some!((_, squad) = world.individual_squad(i.0), continue);

        // FIXME BS NOW: move colors in behavior or somewhere else
        let (r, gg, b) = match behavior.0 {
            oc_individual::behavior::Behavior::Idle(_) => (0.5, 0.5, 0.5),
            oc_individual::behavior::Behavior::Walk(_) => (0.0, 0.0, 1.0),
            oc_individual::behavior::Behavior::Run(_) => (0.0, 0.0, 0.55),
            oc_individual::behavior::Behavior::Crawl(_) => (0.0, 1.0, 0.0),
            oc_individual::behavior::Behavior::Engage(_) => (1.0, 0.0, 0.0),
            oc_individual::behavior::Behavior::Suppress(_) => (0.55, 0.0, 0.0),
            oc_individual::behavior::Behavior::Defend(_) => (1.0, 0.65, 0.0),
            oc_individual::behavior::Behavior::Hide(_) => (1.0, 1.0, 0.0),
            oc_individual::behavior::Behavior::Grenade(_) => (1.0, 0.2, 0.0),
            oc_individual::behavior::Behavior::Smoke(_) => (1.0, 0.5, 0.0),
        };
        // FIXME BS NOW: move alpha in behavior or somewhere else
        let alpha = match squad.leader() == i.0 {
            true => 0.6,
            false => 0.4,
        };
        let color = Color::srgba(r, gg, b, alpha);
        let position = ScreenVec2::from_(position.0, &g.w);
        let position = Vec2::new(position.x, position.y);
        // tracing::trace!(name="spawn-individual-circle", i=?i, position=?position);
        gizmos.circle_2d(position, 3., color);
    }
}

pub fn visibility(
    identity: &Identity,
    i: oc_individual::IndividualIndex,
    individual: &oc_individual::Individual,
    world: &World,
    all: bool,
) -> Visibility {
    match all
        || identity.side == individual.side
        || individual.is_always_visible()
        || world.visible(i)
    {
        true => Visibility::Visible,
        false => Visibility::Hidden,
    }
}

/// Number of distinct z slots available for individuals (for z)
const Z_INDIVIDUAL_SLOTS: u64 = 10_000;
/// Z range (above `Z_INDIVIDUAL`) used to give each individual a distinct z
const Z_INDIVIDUAL_SPAN: f32 = 0.5;

/// Distinct z per individual to avoid z-fighting when sprites overlap
fn z(i: oc_individual::IndividualIndex) -> f32 {
    let slot = (i.0 % Z_INDIVIDUAL_SLOTS) as f32 / Z_INDIVIDUAL_SLOTS as f32;
    Z_INDIVIDUAL + slot * Z_INDIVIDUAL_SPAN
}

#[allow(clippy::too_many_arguments)]
pub fn on_insert_individual(
    individual: On<InsertIndividualEvent>,
    mut commands: Commands,
    g: Res<GameConfig>,
    mut state: ResMut<EntityMapping<oc_individual::IndividualIndex>>,
    mut flashes: ResMut<EntityMapping<MuzzleFlashOf>>,
    animations: Res<SoldierAnimations>,
    weapon_animations: Res<WeaponAnimations>,
    world: Res<crate::world::World>,
    network: Res<crate::network::state::State>,
    config: Res<crate::config::Config>,
) {
    let_some!(g = &g.0, return);
    let_some!(identity = &network.identity, return);
    tracing::trace!(name="spawn-individual", i=?individual.0, position=?individual.1.position);

    let sprite = animations.sprite();
    let gesture = individual.1.gesture.clone();
    let status = individual.1.status;
    let rotation = gesture.rotation(V::Gui);
    let animation = SoldierAnimationInfos::new(individual.1.side, status, gesture);
    let animation = animation.animation(&animations);
    let position = individual.1.position;
    let position_ = ScreenVec2::from_(position, &g.w);
    let all = config.see_all_enemies;
    let visibility = visibility(identity, individual.0, &individual.1, &world, all);
    let volumes = individual
        .1
        .volumes(position, false, &g.w, &g.mod_)
        .clone()
        .into_iter()
        .map(|(p, v, _)| (p, v))
        .collect::<Vec<_>>();

    let entity = commands
        .spawn((
            // Individual properties
            (
                IndividualIndex(individual.0),
                Side(individual.1.side),
                Position(position),
                Tile(individual.1.tile),
                Region(individual.1.region),
                Behavior(individual.1.behavior.clone()),
                Intent(individual.1.intent.clone()),
                Forces(individual.1.forces.clone()),
                Status(individual.1.status),
                Suppress(individual.1.suppress),
                Orders(individual.1.orders.clone()),
                Gesture(individual.1.gesture.clone()),
                Magazines(individual.1.magazines.clone()),
                Weapons(individual.1.weapons.clone()),
            ),
            (
                Direction(individual.1.gesture.direction()),
                Volumes(volumes),
                Material_(individual.1.kind()),
            ),
            // Visual (sprites, animation, etc)
            (
                visibility,
                sprite,
                SpritesheetAnimation::new(animation),
                Transform::from_xyz(position_.x, position_.y, z(individual.0))
                    .with_rotation(rotation),
            ),
            // Surface (clicking, etc)
            (
                Pickable::default(),
                PickRadius(PICK_RADIUS),
                Hovered::default(),
                Selected::default(),
                Dragged::<IndividualIndex>::default(),
            ),
        ))
        .observe(
            on_click
                .run_if(in_state(AppState::InGame))
                .run_if(in_state(InGameState::Battle))
                .run_if(in_state(LeftClickModeType::Select))
                .run_if(in_state(PointerIn::Battle)),
        )
        .observe(
            hover::over
                .run_if(in_state(AppState::InGame))
                .run_if(in_state(InGameState::Battle))
                .run_if(in_state(LeftClickModeType::Select))
                .run_if(in_state(PointerIn::Battle)),
        )
        .observe(
            hover::out
                .run_if(in_state(AppState::InGame))
                .run_if(in_state(InGameState::Battle))
                .run_if(in_state(LeftClickModeType::Select))
                .run_if(in_state(PointerIn::Battle)),
        )
        .observe(
            drag::on_drag_start::<IndividualIndex>
                .run_if(in_state(AppState::InGame))
                .run_if(in_state(InGameState::Battle))
                .run_if(in_state(BattlePhase::Deployment))
                .run_if(in_state(LeftClickModeType::Select))
                .run_if(in_state(PointerIn::Battle)),
        )
        .id();

    if individual.1.side == identity.side {
        commands.entity(entity).with_child((
            IndividualOutline,
            animations.outline_sprite(),
            Transform::from_xyz(0., 0., -0.01),
        ));
    }

    // Weapon overlay
    commands.entity(entity).with_child((
        IndividualWeapon,
        weapon_animations.sprite(WeaponType::Rifle),
        Visibility::Hidden,
        Transform::from_xyz(0., 0., 0.01),
    ));
    let flash = commands
        .spawn((
            IndividualMuzzleFlash::default(),
            weapon_animations.flash_sprite(),
            Anchor::BOTTOM_CENTER,
            Visibility::Hidden,
            Transform::from_xyz(FLASH_X, FLASH_Y, 0.02),
            ChildOf(entity),
        ))
        .id();
    flashes.insert(MuzzleFlashOf(individual.0), flash);

    state.insert(individual.0, entity);
    commands.trigger(RefreshIndividualOrdersEvent(
        individual.0,
        individual.1.orders.clone(),
    ));
}

fn on_click(
    mut event: On<PointerClick>,
    mut commands: Commands,
    mut state: ResMut<super::state::State>,
    world: Res<World>,
    query: Query<&IndividualIndex>,
    mut selected: Query<&mut Selected>,
) {
    let individual = event.original_event_target();
    let_ok!(individual = query.get(individual), return);

    if event.button == PointerButton::Secondary {
        // Unselect other squads: only the right clicked individual's squad stays selected
        selected.iter_mut().for_each(|mut s| s.0 = false);

        let_some!(individual_ = world.individual(individual.0), return);
        let_some!((squad, _) = world.individual_squad(individual.0), return);
        let squads = vec![squad];
        let_some!(squad = world.squad(squad), return);
        let updated = state.update_selected(squads, squad.members.clone(), vec![individual.0]);
        let position = individual_.position;
        commands.trigger(updated);
        tracing::debug!("Trigger open squad contextual menu from individual on {position:?}");
        // Close existing menu before open new one
        commands.trigger(CloseContextMenu::<contextual::Menu>::default());
        commands.trigger(PrepareOpenSquadContextualMenu(position.into()));
        // As its a click, all selected units will be deleted, so we restore them
        commands.trigger(Select::Restore(state.selection()));
    }

    if event.button == PointerButton::Primary {
        commands.trigger(Select::Individual(individual.0));
    }

    event.propagate(false);
}

type RefreshRenderQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Status,
        &'static Side,
        &'static Gesture,
        &'static mut SpritesheetAnimation,
        &'static mut Transform,
        &'static mut Visibility,
    ),
    With<IndividualIndex>,
>;

fn on_refresh_render(
    individual: On<RefreshRender>,
    mut query: RefreshRenderQuery,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
    animations: Res<SoldierAnimations>,
    world: Res<crate::world::World>,
    network: Res<crate::network::state::State>,
    config: Res<crate::config::Config>,
) {
    let_some!(identity = &network.identity, return);
    let i = individual.0;
    let_some!(entity = state.get(&i), return);
    let_some!(individual = world.individual(i), return);
    let Ok((status, side, gesture, mut animation, mut transform, mut visibility_)) =
        query.get_mut(*entity)
    else {
        return;
    };

    let animation_ = SoldierAnimationInfos::new(side.0, status.0, gesture.0.clone());
    let animation_ = animation_.animation(&animations);
    let rotation = gesture.rotation(V::Gui);
    let visibility__ = visibility(identity, i, individual, &world, config.see_all_enemies);

    tracing::trace!(name = "ingame-individual-on-refresh-render", i=?i, animation=?animation_, rotation=?rotation, visibility=?visibility__);
    // Only switch (and thus reset frame/repetition indices) if the target animation actually changed.
    if animation.animation != animation_ {
        animation.switch(animation_);
    }
    transform.rotation = rotation;
    *visibility_ = visibility__;
}

pub fn on_update_individual(update: On<UpdateIndividualEvent>, mut commands: Commands) {
    let (i, update) = (update.0, &update.1);
    tracing::trace!(name="ingame-individual-update", i=?i, update=?update);

    let refresh = trigger_update_event(&mut commands, i, update);
    if refresh {
        commands.trigger(RefreshRender(i));
    }
}

/// World pixels around individual center where cursor hits it
const PICK_RADIUS: f32 = 8.;

pub struct IndividualPlugin;

impl Plugin for IndividualPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugin::<
            oc_individual::IndividualIndex,
            UpdateIndividualPhysicsEvent,
        >::default())
            .add_plugins(HoveredPlugin::<Hover>::default())
            .add_plugins(PickRadiusPlugin)
            .add_plugins(SelectedPlugin::<Selection>::default())
            .init_resource::<EntityMapping<oc_individual::IndividualIndex>>()
            .add_observer(on_insert_individual)
            .add_observer(on_update_individual)
            .add_observer(on_set_position_event)
            .add_observer(on_spawn_individual_phantom)
            .add_observer(on_drop_individual_phantom)
            .add_observer(on_set_behavior_event)
            .add_observer(on_set_intent_event)
            .add_observer(on_set_gesture_event)
            .add_observer(on_set_forces_event)
            .add_observer(on_battle_end)
            .add_observer(on_forgotten_region)
            .add_observer(on_forgot_individual)
            .add_observer(on_set_status_event)
            .add_observer(on_set_orders_event)
            .add_observer(on_set_weapons_event)
            .add_observer(on_accomplished_event)
            .add_observer(on_move_step_accomplished_event)
            .add_observer(on_set_suppress_event)
            .add_observer(on_set_magazines_event)
            .add_observer(on_refresh_render)
            .add_observer(on_prepare_open_squad_contextual_menu)
            .add_systems(
                Update,
                ingame::physics::physics_step::<oc_individual::IndividualIndex, IndividualIndex>
                    .run_if(in_state(AppState::InGame))
                    .run_if(not(in_state(ingame::BattlePhase::End))),
            );

        #[cfg(feature = "debug")]
        {
            use crate::ingame::InGameState;

            app.init_gizmo_group::<PositionsGizmos>();
            app.add_systems(Startup, setup);
            app.add_systems(
                Update,
                positions
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle)),
            );
        }
    }
}

#[derive(Debug, Default)]
pub struct Hover;

impl hover::Hover for Hover {
    fn color() -> Srgba {
        Srgba::new(0.6, 0.8, 1.0, 1.0)
    }

    fn size() -> Vec2 {
        Vec2::splat(10.)
    }
}

#[derive(Debug, Default)]
pub struct Selection;

impl selected::Selection for Selection {
    fn color() -> Srgba {
        bevy::color::palettes::css::BLUE
    }

    fn size() -> Vec2 {
        Vec2::splat(10.)
    }
}

fn on_set_position_event(
    position: On<SetPositionEvent>,
    mut query: Query<&mut Position>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut world: ResMut<crate::world::World>,
    mut commands: Commands,
) {
    let_some!(entity = state.get(&position.0), return);
    let_ok!(mut position_ = query.get_mut(*entity), return);
    let_some!(individual = world.get_individual_mut(position.0), return);
    tracing::trace!(name = "update-individual-position", i=?position.0, position=?position.1);

    position_.0 = position.1;
    individual.position = position.1;

    if let Some((squad_i, squad)) = world.individual_squad(position.0) {
        commands.trigger(crate::ingame::behavior::RefreshSquadsOrdersEvent(
            squad_i,
            squad.orders.clone(),
        ));
    }
}

fn on_set_gesture_event(
    gesture: On<SetGestureEvent>,
    mut query: Query<(&mut Gesture, &mut Direction)>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut world: ResMut<crate::world::World>,
) {
    let_some!(entity = state.get(&gesture.0), return);
    let_ok!((mut gesture_, mut direction) = query.get_mut(*entity), return);
    let_some!(individual = world.get_individual_mut(gesture.0), return);
    tracing::trace!(name = "update-individual-gesture", i=?gesture.0, gesture=?gesture.1);

    gesture_.0 = gesture.1.clone();
    direction.0 = gesture.1.direction();
    individual.gesture = gesture.1.clone();
}

fn on_move_step_accomplished_event(
    accomplished: On<MoveStepAccomplishedEvent>,
    mut query: Query<&mut Intent>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
) {
    let_some!(entity = state.get(&accomplished.0), return);
    let_ok!(mut intent = query.get_mut(*entity), return);
    tracing::trace!(name = "update-individual-move-step-accomplished", i=?accomplished.0);

    intent.0.move_step_accomplished();
}

fn on_accomplished_event(
    accomplished: On<AccomplishedEvent>,
    mut query: Query<&mut Orders>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut commands: Commands,
) {
    let_some!(entity = state.get(&accomplished.0), return);
    let_ok!(mut orders = query.get_mut(*entity), return);
    tracing::trace!(name = "update-individual-accomplished", i=?accomplished.0);

    if !orders.0.is_empty() {
        let order = orders.0.remove(0);
        commands.trigger(DespawnIndividualOrder(accomplished.0, order))
    }
}

fn on_set_orders_event(
    orders: On<SetOrdersEvent>,
    mut query: Query<&mut Orders>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut world: ResMut<crate::world::World>,
    mut commands: Commands,
) {
    let_some!(entity = state.get(&orders.0), return);
    let_ok!(mut orders_ = query.get_mut(*entity), return);
    let_some!(individual = world.get_individual_mut(orders.0), return);
    tracing::trace!(name = "update-individual-set-orders", i=?orders.0, orders=?orders.1);

    orders_.0 = orders.1.clone();
    individual.orders = orders.1.clone();

    tracing::trace!(name = "ingame-behavior-on-set-individual-orders-trigger-refresh-orders", i=?orders.0, events=?orders);
    commands.trigger(RefreshIndividualOrdersEvent(orders.0, orders.1.clone()))
}

/// Battle is ended: individuals stop (no more forces) and become idle
fn on_battle_end(
    phase: On<ingame::SetBattlePhase>,
    individuals: Query<(&IndividualIndex, &Gesture)>,
    mut commands: Commands,
) {
    true_!(phase.phase() == BattlePhase::End, return);

    for (individual, gesture) in individuals {
        let i = individual.0;
        let direction = gesture.direction();

        commands.trigger(SetForcesEvent(i, vec![]));
        commands.trigger(SetBehaviorEvent(
            i,
            oc_individual::behavior::Behavior::Idle(direction),
        ));
        commands.trigger(SetIntentEvent(
            i,
            oc_individual::behavior::Intent::Idle(direction),
        ));
        commands.trigger(SetGestureEvent(
            i,
            oc_individual::Gesture::body(oc_individual::BodyGesture::StandUp(direction)),
        ));
    }
}

fn on_set_status_event(
    status: On<SetStatusEvent>,
    mut query: Query<&mut Status>,
    state: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut orders: Query<&mut Orders>,
    mut world: ResMut<crate::world::World>,
    mut commands: Commands,
) {
    let_some!(entity = state.get(&status.0), return);
    let_ok!(mut status_ = query.get_mut(*entity), return);
    let_some!(individual = world.get_individual_mut(status.0), return);
    tracing::trace!(name = "update-individual-set-status", i=?status.0, status=?status.1);

    status_.0 = status.1;
    individual.status = status.1;

    // Incapacited individual have no more orders
    if !status.1.can_step() {
        individual.orders.clear();
        if let Ok(mut orders) = orders.get_mut(*entity) {
            orders.0.clear();
        }
        commands.trigger(DespawnIndividualOrders(status.0));
    }
}

// TODO: should be automatized (macro? derive ?)
fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    mut state: ResMut<EntityMapping<oc_individual::IndividualIndex>>,
    query: Query<(Entity, &Region, &IndividualIndex)>,
) {
    for (entity, region_, individual) in query {
        let region_: WorldRegionIndex = region_.0;
        if region_ == region.0 {
            tracing::trace!(name = "remove-individual", i=?individual);
            commands.entity(entity).despawn();
            state.remove(&individual.0);
        }
    }
}

pub fn on_forgot_individual(
    individual: On<ForgotIndividual>,
    mut commands: Commands,
    mut individuals: ResMut<EntityMapping<oc_individual::IndividualIndex>>,
    mut flashes: ResMut<EntityMapping<MuzzleFlashOf>>,
) {
    flashes.remove(&MuzzleFlashOf(individual.0));
    if let Some(entity) = individuals.remove(&individual.0) {
        tracing::trace!(name = "remove-individual", i=?individual);
        commands.entity(entity).despawn();
        commands.trigger(DespawnIndividualOrders(individual.0));
    }
}

#[derive(Debug, Event)]
pub struct SpawnIndividualPhantom(Phantom);

#[derive(Debug, Event)]
pub struct DropIndividualPhantom(Entity, WorldVec2);

impl Dragging for IndividualIndex {
    fn spawn(commands: &mut Commands, marker: Phantom) {
        commands.trigger(SpawnIndividualPhantom(marker));
    }

    fn drop(commands: &mut Commands, subject: Entity, _point: WorldVec2, offset: WorldVec2) {
        commands.trigger(DropIndividualPhantom(subject, offset));
    }

    fn visual() -> drag::Visual {
        drag::Visual::Offset
    }
}

pub fn on_spawn_individual_phantom(
    event: On<SpawnIndividualPhantom>,
    mut commands: Commands,
    animations: Res<SoldierAnimations>,
    individuals: Query<(&IndividualIndex, &Side, &Transform)>,
) {
    let phantom = event.0;
    let_ok!(
        (index, side, transform) = individuals.get(phantom.0),
        return
    );

    let mut sprite = animations.sprite();
    sprite.color = Color::srgba(1., 1., 1., 0.6);
    let animation = animations.stand_up(side.0);

    tracing::trace!(name="ingame-individual-on-spawn-individual-phantom", i=?index.0);
    commands.spawn((
        sprite,
        SpritesheetAnimation::new(animation),
        Transform::from_translation(transform.translation),
        phantom,
    ));
    commands.trigger(SetLeftClick(LeftClickMode::Dragging));
}

pub fn on_drop_individual_phantom(
    event: On<DropIndividualPhantom>,
    mut commands: Commands,
    world: Res<World>,
    individuals: Query<&IndividualIndex>,
    zones: Query<&SpawnZone, With<OwnSpawnZone>>,
) {
    let (subject, offset) = (event.0, event.1);
    let_ok!(individual = individuals.get(subject), return);
    let squad = world.individual_squad(individual.0);
    let_some!((squad_i, squad) = squad, return);
    let_some!(leader = world.individual(squad.leader()), return);
    commands.trigger(SetLeftClick(LeftClickMode::Select));

    // Move leader by the drag offset (whatever the dragged squad member), so all dragged squads keep their relative positions.
    let drop = WorldVec2::new(leader.position.x + offset.x, leader.position.y + offset.y);

    if !zones.iter().any(|zone| zone.contains(drop)) {
        tracing::trace!(name="ingame-individual-on-drop-individual-outside-own-spawn", drop=?drop);
        return;
    }

    tracing::trace!(name="ingame-individual-on-drop-individual-set-leader-position", squad=?squad_i, position=?drop);
    let message = SquadMessage::SetLeaderPosition(drop);
    commands.trigger(ToServerEvent(ToServer::Squad(squad_i, message)));
}
