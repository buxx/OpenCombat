use bevy::prelude::*;

use crate::{
    ingame::BattlePhase,
    window::{self, ToggleWindow},
};

const BUTTON_BG: Color = Color::srgb(0.25, 0.25, 0.25);
const BUTTON_BG_HOVER: Color = Color::srgb(0.4, 0.4, 0.4);

#[derive(Component, Default, Clone)]
pub struct BattlePhaseButton;

#[derive(Component, Default, Clone)]
pub struct ActionButtonText;

pub fn actions(world: &crate::world::World, phase: &BattlePhase) -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            height: percent(100),
            flex_shrink: 0.,
        }
        Children [
            BattlePhaseButton
            @battle_state_button(world, phase)
            on(on_battle_phase_click)
            --
            @{ super::morale::morale() }
            --
            @{ super::remaining::remaining() }
        ]
    }
}

fn battle_state_button(_world: &crate::world::World, phase: &BattlePhase) -> impl Scene {
    let label = phase.hud_button_label();

    bsn! {
        Node {
            padding: UiRect::axes(px(12), px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        BackgroundColor(BUTTON_BG)
        on(on_hover)
        on(on_out)
        Children [
            ActionButtonText
            Text(label)
            TextFont { font_size: px(12) }
            Pickable::IGNORE
        ]
    }
}

fn on_hover(e: On<PointerOver>, mut q: Query<&mut BackgroundColor>) {
    if let Ok(mut bg) = q.get_mut(e.entity) {
        bg.0 = BUTTON_BG_HOVER;
    }
}

fn on_out(e: On<PointerOut>, mut q: Query<&mut BackgroundColor>) {
    if let Ok(mut bg) = q.get_mut(e.entity) {
        bg.0 = BUTTON_BG;
    }
}

fn on_battle_phase_click(
    _: On<PointerClick>,
    mut commands: Commands,
    phase: Res<State<BattlePhase>>,
    fight_menu: Res<window::battle::fight::FightWindow>,
) {
    match phase.get() {
        BattlePhase::Deployment => {
            let window = fight_menu
                .0
                .clone()
                .unwrap_or(window::battle::fight::Window);
            commands.trigger(ToggleWindow(window::Window::Fight(window)));
        }
        BattlePhase::End => {}
        BattlePhase::Fight => {
            // FIXME: surrender
            tracing::debug!("Surrender");
        }
    }
}

pub fn update_fight_button(
    phase: Res<State<BattlePhase>>,
    mut text: Single<&mut Text, With<ActionButtonText>>,
) {
    text.0 = phase.hud_button_label().to_string();
}
