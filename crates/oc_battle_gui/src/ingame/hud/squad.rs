use crate::{
    ingame::{camera::GoToPoint, hud::GAP, state::SelectionUpdated},
    sprites::SpriteRect,
    states::GameConfig,
    world::World,
};

use crate::utils::hover::Hovered;
use bevy::prelude::*;
use oc_individual::{Individual, IndividualIndex, Status};
use oc_mod::{Mod, grenade::GrenadeNature, illustration::IllustrationKind, vehicle::CrewRole};
use oc_root::files::Files;
use oc_utils::bevy::EntityMapping;
use oc_utils::{every, let_ok, let_some};
use std::time::Duration;

const ROW_BG: Color = Color::srgb(0.2, 0.25, 0.2);
const STATUS_OPERATIONAL_BG: Color = Color::srgb(0.1, 0.5, 0.15);
const STATUS_STUNNED_BG: Color = Color::srgba(1.0, 0.55, 0.0, 0.3);
const STATUS_DEAD_BG: Color = Color::srgb(0.5, 0., 0.);
pub const HOVER_COLOR: Color = Color::srgb(0.6, 0.8, 1.0);
const ROW_HEIGHT: f32 = 32.;
const COLUMNS_WIDTH: f32 = 200.;
const INDIVIDUAL_ICON_WIDTH: f32 = 30.;
const INDIVIDUAL_ICON_HEIGHT: f32 = 30.;

#[derive(Component, Default, Clone)]
pub struct SquadDetails;

#[derive(Component)]
pub struct IndividualRow(pub IndividualIndex);

#[derive(Component, Default, Clone)]
pub struct IndividualGrenadesText(pub Option<IndividualIndex>);

#[derive(Component, Default, Clone)]
pub struct IndividualNameText(pub Option<IndividualIndex>);

#[derive(Component, Default, Clone)]
pub struct IndividualStatusText(pub Option<IndividualIndex>);

#[derive(Component, Default, Clone)]
pub struct IndividualKiaText(pub Option<IndividualIndex>);

#[derive(Component, Default, Clone)]
pub struct IndividualRoundsText(pub Option<IndividualIndex>);

pub fn squad() -> impl Scene {
    bsn! {
        SquadDetails
        Node {
            width: px(COLUMNS_WIDTH),
            height: percent(100),
            flex_shrink: 0.,
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP / 2.),
            overflow: Overflow::scroll_y(),
        }
    }
}

fn row(
    g: &oc_network::GameConfig,
    files: &Files,
    world: &World,
    i: IndividualIndex,
    individual: &Individual,
) -> impl Scene {
    let sprites = files.sprites();
    let sprite = sprites.join("illustrations.png");
    let kind = IllustrationKind::IngameIndividual;
    let illustration = g.mod_.illustration(kind, individual.illustration);
    let rect = illustration.inner().rect();
    let action = status_label(world, individual);
    let kia_visibility = kia_visibility(individual.status);
    let rounds = rounds_label(&g.mod_, world, individual);
    let status_color = status_color(individual.status);
    let grenades = grenades_label(&g.mod_, individual);

    bsn! {
        Node {
            width: percent(100),
            height: px(ROW_HEIGHT),
            flex_shrink: 0.,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Start,
            padding: UiRect::horizontal(px(2)),
        }
        BackgroundColor(ROW_BG)
        on(on_hover)
        on(on_out)
        on(on_click)
        Children [
            // Icon
            Node { width: px(INDIVIDUAL_ICON_WIDTH), height: px(INDIVIDUAL_ICON_HEIGHT), flex_shrink: 0. }
            ImageNode { image: sprite, rect: {Some(rect)} }
            Pickable::IGNORE
            Children [
                IndividualKiaText(i)
                Node { position_type: PositionType::Absolute, left: px(0), top: px(8) }
                Text("K.I.A")
                TextFont { font_size: px(11) }
                TextColor(Color::srgb(1.0, 0.1, 0.1))
                kia_visibility
                Pickable::IGNORE
            ]
            --
            // Name and status
            Node { flex_direction: FlexDirection::Column }
            Pickable::IGNORE
            Children [
                // Name
                IndividualNameText(i)
                Text({format!("Individual {}", i.0)})
                TextFont { font_size: px(11) }
                BackgroundColor(status_color)
                Pickable::IGNORE
                --
                // Status
                IndividualStatusText(i)
                Text(action)
                TextFont { font_size: px(11) }
                TextColor(Color::srgb(0.3, 0.5, 1.0))
                Pickable::IGNORE
            ]
            --
            // Ammunition count
            Node { flex_direction: FlexDirection::Column }
            Pickable::IGNORE
            Children [
                IndividualRoundsText(i)
                Text(rounds)
                TextFont { font_size: px(10) }
                Pickable::IGNORE
                --
                IndividualGrenadesText(i)
                Text(grenades)
                TextFont { font_size: px(10) }
                Pickable::IGNORE
            ]
        ]
    }
}

pub fn spawn_rows(
    selection: On<SelectionUpdated>,
    mut commands: Commands,
    world: Res<World>,
    mut details: Query<(Entity, &mut ScrollPosition), With<SquadDetails>>,
    rows: Query<Entity, With<IndividualRow>>,
    g: Res<GameConfig>,
    files: Res<crate::states::GuiFiles>,
) {
    let_some!(g = &g.0, return);
    let_some!(files = &files.0, return);

    for row in &rows {
        commands.entity(row).despawn();
    }

    let_ok!((details, mut scroll) = details.single_mut(), return);
    scroll.y = 0.;
    let_some!(squad = selection.selected_squads.first().copied(), return);
    let_some!(squad = world.squad(squad), return);

    for i in squad.members.iter().copied() {
        let_some!(individual = world.individual(i), continue);

        commands
            .spawn_scene(row(g, files, &world, i, individual))
            .insert((IndividualRow(i), ChildOf(details)));
    }
}

type StatusTexts<'w, 's> = Query<
    'w,
    's,
    (&'static IndividualStatusText, &'static mut Text),
    (
        Without<IndividualRoundsText>,
        Without<IndividualGrenadesText>,
    ),
>;
type RoundsTexts<'w, 's> = Query<
    'w,
    's,
    (&'static IndividualRoundsText, &'static mut Text),
    (
        Without<IndividualStatusText>,
        Without<IndividualGrenadesText>,
    ),
>;
type GrenadesTexts<'w, 's> = Query<
    'w,
    's,
    (&'static IndividualGrenadesText, &'static mut Text),
    (Without<IndividualStatusText>, Without<IndividualRoundsText>),
>;

#[allow(clippy::too_many_arguments)]
pub fn update_rows(
    world: Res<World>,
    g: Res<GameConfig>,
    mut texts: StatusTexts,
    mut rounds: RoundsTexts,
    mut grenades: GrenadesTexts,
    mut names: Query<(&IndividualNameText, &mut BackgroundColor)>,
    mut kias: Query<(&IndividualKiaText, &mut Visibility)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    let_some!(g = &g.0, return);
    every!(timer, time, Duration::from_secs(1));

    for (individual, mut text) in &mut texts {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.individual(individual), continue);
        text.0 = status_label(&world, individual);
    }

    for (individual, mut visibility) in &mut kias {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.individual(individual), continue);
        *visibility = kia_visibility(individual.status);
    }

    for (individual, mut bg) in &mut names {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.individual(individual), continue);
        bg.0 = status_color(individual.status);
    }

    for (individual, mut text) in &mut rounds {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.individual(individual), continue);
        text.0 = rounds_label(&g.mod_, &world, individual);
    }

    for (individual, mut text) in &mut grenades {
        let_some!(individual = individual.0, continue);
        let_some!(individual = world.individual(individual), continue);
        text.0 = grenades_label(&g.mod_, individual);
    }
}

fn grenades_label(mod_: &Mod, individual: &Individual) -> String {
    let (mut frag, mut smoke) = (0, 0);
    for grenade in &individual.weapons.grenades {
        match mod_.grenade(*grenade).nature {
            GrenadeNature::Frag => frag += 1,
            GrenadeNature::Smoke => smoke += 1,
        }
    }
    format!("gr. {frag} sm. {smoke}")
}

fn on_hover(
    e: On<PointerOver>,
    mut commands: Commands,
    rows: Query<&IndividualRow>,
    individuals: Res<EntityMapping<IndividualIndex>>,
    mut hovered: Query<&mut Hovered>,
) {
    let_ok!(row = rows.get(e.entity), return);
    commands
        .entity(e.entity)
        .insert(Outline::new(px(1), Val::ZERO, HOVER_COLOR));
    let_some!(entity = individuals.get(&row.0), return);
    let_ok!(mut h = hovered.get_mut(*entity), return);
    **h = true;
}

fn on_click(
    e: On<PointerClick>,
    mut commands: Commands,
    rows: Query<&IndividualRow>,
    world: Res<World>,
) {
    if e.button != PointerButton::Primary {
        return;
    }
    let_ok!(row = rows.get(e.entity), return);
    let_some!(individual = world.individual(row.0), return);
    commands.trigger(GoToPoint(individual.position.into()));
}

fn on_out(
    e: On<PointerOut>,
    mut commands: Commands,
    rows: Query<&IndividualRow>,
    individuals: Res<EntityMapping<IndividualIndex>>,
    mut hovered: Query<&mut Hovered>,
) {
    let_ok!(row = rows.get(e.entity), return);
    commands.entity(e.entity).remove::<Outline>();
    let_some!(entity = individuals.get(&row.0), return);
    let_ok!(mut h = hovered.get_mut(*entity), return);
    **h = false;
}

fn status_label(world: &World, individual: &Individual) -> String {
    match individual.status {
        Status::Dead => String::new(),
        Status::Operational => match individual.embarked {
            Some((vehicle, role)) => crew_label(world, vehicle, role).to_string(),
            None => individual.action_hud_label().to_string(),
        },
        Status::Stunned(_) => individual.status.hud_label().to_string(),
    }
}

/// What does an embarked individual according to its crew `role` and its `vehicle` state
fn crew_label(world: &World, vehicle: oc_vehicle::VehicleIndex, role: CrewRole) -> &'static str {
    let_some!(vehicle_ = world.vehicle(vehicle), return "");
    // Without operational loader, gunner loads the gun himself
    let gunner_loads = !world.vehicle_crew(vehicle).iter().any(|i| {
        world.individual(*i).is_some_and(|i| {
            i.embarked.is_some_and(|(_, r)| r == CrewRole::Loader) && i.status.is_target()
        })
    });
    vehicle_.crew_hud_label(role, gunner_loads)
}

/// Primary weapon rounds or, when embarked as gunner, shells stored in the vehicle
fn rounds_label(mod_: &Mod, world: &World, individual: &Individual) -> String {
    match individual.embarked {
        Some((vehicle, CrewRole::Gunner)) => world
            .vehicle(vehicle)
            .map(|v| format!("{} shells", v.shells_count()))
            .unwrap_or_default(),
        _ => format!("{} rnds.", primary_rounds(mod_, individual)),
    }
}

fn kia_visibility(status: Status) -> Visibility {
    if matches!(status, Status::Dead) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

fn status_color(status: Status) -> Color {
    match status {
        Status::Operational => STATUS_OPERATIONAL_BG,
        Status::Stunned(_) => STATUS_STUNNED_BG,
        Status::Dead => STATUS_DEAD_BG,
    }
}

fn primary_rounds(mod_: &Mod, individual: &Individual) -> u32 {
    let_some!(primary = &individual.weapons.primary, return 0);
    let weapon = mod_.weapon(primary.i);
    let compatibles = weapon.magazines();
    let carried: u32 = individual
        .magazines
        .iter()
        .filter_map(|carried| compatibles.iter().find(|m| m.index() == *carried))
        .map(|m| m.capacity() as u32)
        .sum();

    primary.filled_count as u32 + carried
}
