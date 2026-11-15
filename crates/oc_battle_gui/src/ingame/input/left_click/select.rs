use bevy::prelude::*;
use oc_individual::squad::SquadIndex;
use oc_physics::update::bevy::Position;
use oc_root::WcfgFrom;
use oc_root::geo::{ScreenVec2, WorldVec2};
use oc_utils::bevy::EntityMapping;
use oc_utils::{let_ok, let_some};
use rustc_hash::FxHashSet;

use crate::cursor_to;
use crate::entity::individual::IndividualIndex;
use crate::ingame::squad::menu::contextual;
use crate::ingame::state::{Selection, State};
use crate::menu::contextual::close::CloseContextMenu;
use crate::states::GameConfig;
use crate::utils::selected::Selected;
use crate::world::World;

#[derive(Debug, Clone, Event)]
pub enum Select {
    Individual(oc_individual::IndividualIndex),
    Restore(Selection),
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct AreaGizmos;

pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-input-left-click-select-setup");
    let (gizmos, _) = config.config_mut::<AreaGizmos>();
    gizmos.line.width = 1.0;
}

#[allow(clippy::too_many_arguments)]
pub fn area(
    g: Res<GameConfig>,
    mut state: ResMut<crate::ingame::input::State>,
    mut ingame: ResMut<crate::ingame::state::State>,
    buttons: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut gizmos: Gizmos<AreaGizmos>,
    mut query: Query<(&IndividualIndex, &Position, &mut Selected)>,
    world: Res<crate::world::World>,
    mapping: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);

    // Area start to exist from left click
    if buttons.pressed(MouseButton::Left) {
        // Save the original position only if it's first time we press left click
        if state.first_left_press.is_none() {
            let_some!(position = window.cursor_position(), return);
            let position = Vec2::new(position.x, position.y);
            let position = cursor_to!(position, camera, &g.w, WorldVec2);

            state.first_left_press = Some(position);
        }
    }

    // Manage area during the left click press
    if let Some(start) = state.first_left_press {
        let start_ = ScreenVec2::from_(start, &g.w);
        let start_ = Vec2::new(start_.x, start_.y);

        let_some!(end = window.cursor_position(), return);
        let end = Vec2::new(end.x, end.y);
        let (camera, transform) = *camera;
        let end = camera.viewport_to_world_2d(transform, end);
        let_ok!(end = end, return);
        let end_ = WorldVec2::from_(end, &g.w);

        let size = (end - start_).abs();
        let center = (start_ + end) * 0.5;

        gizmos.rect_2d(center, size, bevy::color::palettes::css::BLACK);

        // Compute selection when left click is released
        if buttons.just_released(MouseButton::Left) {
            if end_ != start {
                let mut individuals = vec![];

                for (i, position, _) in &query {
                    let position = position.0;

                    // Avoid rectangle direction by taking min and max
                    let min_x = start.x.min(end_.x);
                    let max_x = start.x.max(end_.x);
                    let min_y = start.y.min(end_.y);
                    let max_y = start.y.max(end_.y);

                    if position.x >= min_x
                        && position.x <= max_x
                        && position.y >= min_y
                        && position.y <= max_y
                    {
                        individuals.push(i.0);
                    }
                }

                // We want to know which squad are concerned by selected individuals
                let squads = individuals
                    .iter()
                    .filter_map(|individual| world.individual_squad(*individual))
                    .map(|(squad, _)| squad)
                    .collect::<FxHashSet<SquadIndex>>();
                let squad_members: Vec<oc_individual::IndividualIndex> = squads
                    .iter()
                    .filter_map(|squad| world.squad(*squad))
                    .flat_map(|squad| squad.members.clone())
                    .collect();
                let squads: Vec<SquadIndex> = squads.into_iter().collect();

                // Area selection replaces previous selection. Don't rely on `unselect` observer: it is not
                // triggered when press and release are not on the same entity (ex. release on a spawn zone).
                for (_, _, mut selected) in &mut query {
                    selected.0 = false;
                }

                // Set to "selected" the "Selected" component
                for individual in &squad_members {
                    if let Some(entity) = mapping.get(individual)
                        && let Ok((_, _, mut selected)) = query.get_mut(*entity)
                    {
                        selected.0 = true;
                    }
                }

                // Update the state too about who is selected
                let updated = ingame.update_selected(squads, squad_members, vec![]);
                commands.trigger(updated);
            }
            state.first_left_press = None;
        }
    }
}

/// Un select all. Observer which select must stop propagation to avoid execute this observer.
pub fn unselect(click: On<PointerClick>, mut state: ResMut<State>, mut commands: Commands) {
    if click.button == PointerButton::Primary {
        let updated = state.update_selected(vec![], vec![], vec![]);
        commands.trigger(updated);
        commands.trigger(CloseContextMenu::<contextual::Menu>::default());
    }
}

pub fn on_select(
    event: On<Select>,
    world: Res<World>,
    mut state: ResMut<State>,
    individuals: Res<EntityMapping<oc_individual::IndividualIndex>>,
    mut query: Query<(&IndividualIndex, &mut Selected)>,
    mut commands: Commands,
) {
    match event.clone() {
        Select::Individual(i) => select_individual(
            i,
            &world,
            &mut state,
            &individuals,
            &mut query,
            &mut commands,
        ),
        Select::Restore(selection) => select_restore(
            selection,
            &mut state,
            &individuals,
            &mut query,
            &mut commands,
        ),
    }
}

fn select_individual(
    i: oc_individual::IndividualIndex,
    world: &World,
    state: &mut State,
    individuals: &EntityMapping<oc_individual::IndividualIndex>,
    query: &mut Query<(&IndividualIndex, &mut Selected)>,
    commands: &mut Commands,
) {
    let_some!((squad, _) = world.individual_squad(i), return);
    let squads = vec![squad];
    let_some!(squad = world.squad(squad), return);

    let updated = state.update_selected(squads, squad.members.clone(), vec![i]);
    commands.trigger(updated);

    for individual in &squad.members {
        let_some!(individual = individuals.get(individual), continue);
        let_ok!((_, mut selected) = query.get_mut(*individual), continue);
        selected.0 = true;
    }
}

fn select_restore(
    selection: Selection,
    state: &mut State,
    individuals: &EntityMapping<oc_individual::IndividualIndex>,
    query: &mut Query<(&IndividualIndex, &mut Selected)>,
    commands: &mut Commands,
) {
    let updated = state.update_selected(
        selection.selected_squads.clone(),
        selection.selected_squads_individuals.clone(),
        selection.selected_individuals.clone(),
    );
    commands.trigger(updated);

    for individual in [
        selection.selected_squads_individuals.clone(),
        selection.selected_individuals.clone(),
    ]
    .concat()
    {
        let_some!(individual = individuals.get(&individual), continue);
        let_ok!((_, mut selected) = query.get_mut(*individual), continue);
        selected.0 = true;
    }
}
