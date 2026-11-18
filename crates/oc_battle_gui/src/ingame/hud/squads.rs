use crate::utils::selected::Selected;
use crate::{
    ingame::{self, camera::GoToPoint, hud::SCROLL_LINE_PX},
    network,
    sprites::SpriteRect,
    states::GameConfig,
};
use crate::{
    ingame::{hud::GAP, state::SelectionUpdated},
    world::World,
};

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
};
use oc_individual::squad::{Squad, SquadIndex, SquadStatus};
use oc_mod::illustration::IllustrationKind;
use oc_root::files::Files;
use oc_utils::bevy::EntityMapping;
use oc_utils::{every, let_ok, let_some, true_};
use std::time::Duration;

const CELL_BG: Color = Color::srgb(0.2, 0.25, 0.2);
const CELL_BG_HOVER: Color = Color::srgb(0.35, 0.45, 0.35);

const CELL_OUTLINE_SELECTED: Color = Color::srgb(0.2, 0.4, 1.0);

const STATUS_BG_OPERATIONAL: Color = Color::srgb(0.1, 0.5, 0.1);
const STATUS_BG_CASUALITIES: Color = Color::srgb(0.7, 0.6, 0.05);
const STATUS_BG_PARTIALLY_DESTROYED: Color = Color::srgb(0.7, 0.1, 0.1);
const STATUS_BG_DESTROYED: Color = Color::srgb(0.3, 0., 0.);

const SQUAD_CELL_WIDTH: f32 = 122.;
const SQUAD_CELL_HEIGHT: f32 = 52.;
const SQUAD_ICON_WIDTH: f32 = 50.;
const SQUAD_ICON_HEIGHT: f32 = 50.;

#[derive(Component, Default, Clone)]
pub struct SquadGrid;

#[derive(Component, Default, Clone)]
pub struct SquadNameText(pub Option<SquadIndex>);

#[derive(Component, Default, Clone)]
pub struct SquadBehaviorText(pub Option<SquadIndex>);

#[derive(Component)]
pub struct SquadCell(pub SquadIndex);

pub fn squads_column() -> impl Scene {
    bsn! {
        Node {
            flex_grow: 1.,
            height: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(GAP),
        }
        Children [
            @{ squads() }
            --
            @{ super::supports::supports() }
        ]
    }
}

fn squads() -> impl Scene {
    bsn! {
        SquadGrid
        Node {
            flex_grow: 1.,
            min_height: px(0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexStart,
            row_gap: px(GAP),
            column_gap: px(GAP),
            overflow: Overflow::scroll_y(),
        }
    }
}

fn status_color(status: &SquadStatus) -> Color {
    match status {
        SquadStatus::Operational => STATUS_BG_OPERATIONAL,
        SquadStatus::Casualities => STATUS_BG_CASUALITIES,
        SquadStatus::PartiallyDestroyed => STATUS_BG_PARTIALLY_DESTROYED,
        SquadStatus::Destroyed => STATUS_BG_DESTROYED,
    }
}

fn squad_cell(
    g: &oc_network::GameConfig,
    files: &Files,
    i: SquadIndex,
    squad: &Squad,
) -> impl Scene {
    let sprites = files.sprites();
    let sprite = sprites.join("illustrations.png");
    let kind = IllustrationKind::IngameSquad;
    let illustration = g.mod_.illustration(kind, squad.illustration);
    let rect = illustration.inner().rect();

    bsn! {
        Node {
            width: px(SQUAD_CELL_WIDTH),
            height: px(SQUAD_CELL_HEIGHT),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(4),
            padding: UiRect::all(px(4)),
        }
        BackgroundColor(CELL_BG)
        on(on_hover)
        on(on_out)
        on(on_click)
        Children [
            // Squad icon
                Node { width: px(SQUAD_ICON_WIDTH), height: px(SQUAD_ICON_HEIGHT), flex_shrink: 0. }
                ImageNode { image: sprite, rect: {Some(rect)} }
                Pickable::IGNORE
            --
            // Squad name & behavior
            Node { flex_direction: FlexDirection::Column }
            Pickable::IGNORE
            Children [
                SquadNameText(i)
                Text({format!("Squad {}", i.0)})
                TextFont { font_size: px(14) }
                Pickable::IGNORE
                BackgroundColor(status_color(&squad.status))
                --
                SquadBehaviorText(i)
                Text("Behavior")
                TextFont { font_size: px(14) }
                Pickable::IGNORE
                TextColor(Color::srgb(0.3, 0.5, 1.0))
            ]
        ]
    }
}

pub fn spawn_squad_cells(
    mut commands: Commands,
    world: Res<World>,
    grid: Query<Entity, With<SquadGrid>>,
    cells: Query<(), With<SquadCell>>,
    g: Res<GameConfig>,
    files: Res<crate::states::GuiFiles>,
    network: Res<network::state::State>,
) {
    let_some!(g = &g.0, return);
    let_some!(files = &files.0, return);
    let Ok(grid) = grid.single() else { return };
    true_!(cells.is_empty() && !world.squads_refs.is_empty(), return);

    let_some!(identity = &network.identity, return);

    for i in world.squads() {
        let_some!(squad = world.squad(i), continue);
        if squad.side != identity.side {
            continue;
        }
        commands
            .spawn_scene(squad_cell(g, files, i, squad))
            .insert((SquadCell(i), ChildOf(grid)));
    }
}

pub fn outline_selected_cells(
    selection: On<SelectionUpdated>,
    mut commands: Commands,
    cells: Query<(Entity, &SquadCell)>,
) {
    for (entity, cell) in &cells {
        if selection.selected_squads.contains(&cell.0) {
            commands
                .entity(entity)
                .insert(Outline::new(px(1), Val::ZERO, CELL_OUTLINE_SELECTED));
        } else {
            commands.entity(entity).remove::<Outline>();
        }
    }
}

fn on_hover(e: On<PointerOver>, mut q: Query<&mut BackgroundColor>) {
    if let Ok(mut bg) = q.get_mut(e.entity) {
        bg.0 = CELL_BG_HOVER;
    }
}

fn on_out(e: On<PointerOut>, mut q: Query<&mut BackgroundColor>) {
    if let Ok(mut bg) = q.get_mut(e.entity) {
        bg.0 = CELL_BG;
    }
}

fn on_click(
    e: On<PointerClick>,
    mut commands: Commands,
    cells: Query<&SquadCell>,
    world: Res<World>,
    mut state: ResMut<ingame::state::State>,
    individuals: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut selected: Query<&mut Selected>,
) {
    let_ok!(cell = cells.get(e.entity), return);
    let i = cell.0;
    let_some!(squad = world.squad(i), return);
    let updated = state.update_selected(vec![i], squad.members.clone(), vec![]);
    commands.trigger(updated);

    // Unselect other
    for mut s in &mut selected {
        s.0 = false;
    }

    // Select member of squad
    for member in &squad.members {
        let_some!(entity = individuals.get(member), continue);
        let_ok!(mut s = selected.get_mut(*entity), continue);
        s.0 = true;
    }

    if e.count >= 2 {
        commands.trigger(GoToPoint(squad.position));
    }
}

pub fn update_squad_behaviors(
    world: Res<World>,
    mut texts: Query<(&SquadBehaviorText, &mut Text)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    every!(timer, time, Duration::from_secs(1));

    for (squad, mut text) in &mut texts {
        let_some!(squad = squad.0, continue);
        let_some!(squad = world.squad(squad), continue);
        text.0 = if matches!(squad.status, SquadStatus::Destroyed) {
            String::new()
        } else {
            squad
                .orders
                .first()
                .map(|o| o.hud_label().to_string())
                .unwrap_or_default()
        };
    }
}

pub fn update_squad_status_colors(
    world: Res<World>,
    mut texts: Query<(&SquadNameText, &mut BackgroundColor)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    every!(timer, time, Duration::from_secs(1));

    for (name, mut bg) in &mut texts {
        let_some!(i = name.0, continue);
        let_some!(squad = world.squad(i), continue);
        let color = status_color(&squad.status);
        if bg.0 != color {
            bg.0 = color;
        }
    }
}

pub fn scroll_squad_grid(
    mut wheel: MessageReader<MouseWheel>,
    mut grid: Query<(&mut ScrollPosition, &ComputedNode, &UiGlobalTransform), With<SquadGrid>>,
    window: Single<&Window>,
) {
    let_some!(cursor = window.physical_cursor_position(), return);
    let Ok((mut scroll, computed, transform)) = grid.single_mut() else {
        return;
    };
    if !computed.contains_point(*transform, cursor) {
        return;
    }

    for event in wheel.read() {
        let dy = match event.unit {
            MouseScrollUnit::Line => event.y * SCROLL_LINE_PX,
            MouseScrollUnit::Pixel => event.y,
        };
        let max = (computed.content_size().y - computed.size().y).max(0.)
            * computed.inverse_scale_factor();
        scroll.y = (scroll.y - dy).clamp(0., max);
    }
}
