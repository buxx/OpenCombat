use bevy::prelude::*;
use bevy_egui::EguiContexts;
use oc_root::geo::WorldVec2;
use oc_utils::{let_ok, let_some};

use crate::{
    ingame::{InGameState, hud::Hud, input::left_click::LeftClickModeType},
    states::{AppState, PointerIn},
};

pub mod client;
pub mod individual;
pub mod keyboard;
pub mod left_click;
pub mod projectile;

#[derive(Debug, Resource, Default)]
pub struct State {
    pub clicks: Vec<WorldVec2>,
    pub first_left_press: Option<WorldVec2>,
    /// Window cursor position when right button was pressed
    pub first_right_press: Option<Vec2>,
    /// Right press moved enough to be considered as a map drag (and no more as a click)
    pub right_dragging: bool,
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<LeftClickModeType>()
            .init_gizmo_group::<left_click::select::AreaGizmos>()
            .init_resource::<State>()
            .init_resource::<left_click::LeftClick>()
            .init_resource::<left_click::support::RequestedSupports>()
            .add_observer(client::on_to_client)
            .add_observer(left_click::on_set_left_click)
            .add_observer(left_click::on_spawn_clicks_line)
            .add_observer(left_click::on_despawn_clicks_line)
            .add_observer(left_click::select::on_select)
            .add_observer(
                individual::on_click
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Select)),
            )
            .add_observer(
                left_click::select::unselect
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Select))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_observer(
                left_click::order::on_click
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Order))
                    .run_if(in_state(PointerIn::Battle))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_observer(
                left_click::support::on_click
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Support))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_observer(
                left_click::order::on_set_left_click
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(PointerIn::Battle))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(Startup, left_click::select::setup)
            .add_systems(
                Update,
                (watch_clicks,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(PointerIn::Battle))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                Update,
                (left_click::select::area,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Select))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                Update,
                (keyboard::on_key_press,).run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                Update,
                (left_click::order::system)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Order))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                Update,
                (left_click::order::cursor_sprite)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Order))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                OnExit(LeftClickModeType::Order),
                left_click::order::despawn_cursor_sprite,
            )
            .add_systems(
                Update,
                (left_click::lov::system)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::LineOfView))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                Update,
                (left_click::support::system)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Support))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                Update,
                left_click::support::draw_requested
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle)),
            )
            .add_systems(Update, pointer_in);

        #[cfg(feature = "debug")]
        app.init_resource::<left_click::SpawnProjectileLeftClick>()
            .add_systems(
                Update,
                (left_click::update_spawn_projectile_clicks_line,)
                    .run_if(in_state(AppState::InGame)),
            )
            .add_observer(left_click::on_set_spawn_projectile_left_click)
            .add_systems(
                Update,
                (left_click::spawn_projectile::system)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::SpawnProjectile))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                Update,
                (left_click::explosion::system)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(LeftClickModeType::Explosion))
                    .run_if(in_state(PointerIn::Battle)),
            );
    }
}

fn watch_clicks(
    mut state: ResMut<State>,
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        state.first_right_press = window.cursor_position();
        state.right_dragging = false;
    }
    if buttons.just_released(MouseButton::Right) {
        state.first_right_press = None;
        state.right_dragging = false;
    }
}

fn pointer_in(
    mut contexts: EguiContexts,
    mut pointer: ResMut<NextState<PointerIn>>,
    window: Single<&Window>,
    hud: Single<(&ComputedNode, &UiGlobalTransform), With<Hud>>,
) {
    let (node, transform) = *hud;
    let_some!(cursor = window.physical_cursor_position(), return);
    let_ok!(contexts = contexts.ctx_mut(), return);

    if contexts.is_pointer_over_egui() {
        *pointer = NextState::Pending(PointerIn::Window);
        return;
    }

    if node.contains_point(*transform, cursor) {
        *pointer = NextState::Pending(PointerIn::Hud);
        return;
    }

    *pointer = NextState::Pending(PointerIn::Battle);
}
