use bevy::prelude::*;

use crate::{ingame::BattlePhase, states::AppState};

pub mod actions;
pub mod minimap;
pub mod morale;
pub mod remaining;
pub mod squad;
pub mod squads;
pub mod supports;

const HUD_HEIGHT: f32 = 200.;
const GAP: f32 = 4.;
const SCROLL_LINE_PX: f32 = 20.;
const HUD_BG: Color = Color::srgba(0., 0., 0., 0.95);

#[derive(Component, Default, Clone)]
pub struct Hud;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_hud)
            .add_systems(
                Update,
                (
                    squads::spawn_squad_cells,
                    supports::spawn_support_buttons,
                    squads::scroll_squad_grid,
                    squads::update_squad_behaviors,
                    squads::update_squad_status_colors,
                    squad::update_rows,
                    remaining::tick_remaining,
                    minimap::load_image,
                    minimap::update_squad_markers,
                    minimap::update_viewport,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(BattlePhase::Fight), actions::update_fight_button)
            .init_resource::<remaining::Remaining>()
            .add_observer(supports::on_set_support_count)
            .add_observer(supports::on_support_executed)
            .add_observer(squad::spawn_rows)
            .add_observer(squads::outline_selected_cells)
            .add_observer(morale::on_update_morale)
            .add_observer(remaining::on_update_remaining);
    }
}

fn spawn_hud(
    mut commands: Commands,
    existing: Query<(), With<Hud>>,
    world: Res<crate::world::World>,
    phase: Res<State<BattlePhase>>,
) {
    if existing.is_empty() {
        commands.spawn_scene(hud(&world, &phase));
    }
}

fn hud(world: &crate::world::World, phase: &BattlePhase) -> impl Scene {
    bsn! {
        Hud
        Node {
            position_type: PositionType::Absolute,
            bottom: px(0),
            left: px(0),
            width: percent(100),
            height: px(HUD_HEIGHT),
            padding: UiRect::all(px(GAP)),
            column_gap: px(GAP),
        }
        BackgroundColor(HUD_BG)
        Children [
            @{ squads::squads_column() }
            --
            @{ squad::squad() }
            --
            @{ actions::actions(world, phase) }
            --
            @{ minimap::minimap() }
        ]
    }
}
