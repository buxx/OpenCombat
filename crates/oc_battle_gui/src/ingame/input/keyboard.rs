use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::ingame::camera::WindowResizeWhenWorldMap;
#[cfg(feature = "debug")]
use crate::ingame::camera::debug::tile::ToggleShowTiles;
use crate::ingame::camera::map::SaveCurrentWindowCenterAsBattleCenter;
use crate::ingame::input::left_click::{LeftClickMode, SetLeftClick};
use crate::ingame::lov::SpawnLovConfig;
use crate::ingame::squad::menu::contextual;
use crate::ingame::{
    BattlePhase, QuitHeightMap, RestoreBattleCenter, SwitchToBattleMap, SwitchToWorldMap,
};
use crate::ingame::{SwitchToHeightMap, camera};
use crate::menu::contextual::{ContextMenu, close::CloseContextMenu};
use crate::window;
use crate::window::ToggleWindow;
#[cfg(feature = "debug")]
use crate::window::Window;
#[cfg(feature = "debug")]
use crate::window::debug::battle::DebugBattleWindow;

#[allow(clippy::too_many_arguments)]
pub fn on_key_press(
    mut commands: Commands,
    mut keyboard: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    camera: Res<camera::State>,
    fight_menu: Res<window::battle::fight::FightWindow>,
    battle_menu: Res<window::menu::battle::BattleMenuWindow>,
    phase: Res<State<BattlePhase>>,
    context_menus: Query<(), With<ContextMenu<contextual::Menu>>>,
    #[cfg(feature = "debug")] debug: Res<DebugBattleWindow>,
) {
    let ctrl = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);

    for event in keyboard.read() {
        match (event.state, event.key_code) {
            (ButtonState::Released, KeyCode::KeyV) => match camera.focus {
                camera::Focus::Battle => {
                    // TODO: display something to show to player start_z_plus (standup, crouched, ...)
                    let config = SpawnLovConfig::default();
                    commands.trigger(SetLeftClick(LeftClickMode::LineOfView(config)));
                }
                camera::Focus::Height | camera::Focus::World => {}
            },
            (ButtonState::Released, KeyCode::F1) => match camera.focus {
                camera::Focus::Battle => {
                    tracing::debug!("Trigger switch to world map (from battle map)");
                    commands.trigger(SaveCurrentWindowCenterAsBattleCenter);
                    commands.trigger(SwitchToWorldMap);
                    commands.trigger(WindowResizeWhenWorldMap);
                }
                camera::Focus::Height => {}
                camera::Focus::World => {
                    tracing::debug!("Trigger switch to battle map (from world map)");
                    commands.trigger(RestoreBattleCenter);
                    commands.trigger(SwitchToBattleMap);
                }
            },
            (ButtonState::Released, KeyCode::F2) => match camera.focus {
                camera::Focus::Battle => {
                    tracing::debug!("Trigger switch to height map (from battle map)");
                    commands.trigger(SaveCurrentWindowCenterAsBattleCenter);
                    commands.trigger(SwitchToHeightMap);
                }
                camera::Focus::Height => {
                    tracing::debug!("Trigger switch to battle map (from height map)");
                    commands.trigger(QuitHeightMap);
                    commands.trigger(RestoreBattleCenter);
                    commands.trigger(SwitchToBattleMap);
                }
                camera::Focus::World => {}
            },
            (ButtonState::Released, KeyCode::KeyF)
                if ctrl && matches!(phase.get(), BattlePhase::Deployment) =>
            {
                let window = fight_menu
                    .0
                    .clone()
                    .unwrap_or(window::battle::fight::Window);
                commands.trigger(ToggleWindow(window::Window::Fight(window)));
            }
            (ButtonState::Released, KeyCode::Escape) => {
                // Escape only closes contextual menus when there are some
                if !context_menus.is_empty() {
                    commands.trigger(CloseContextMenu::<contextual::Menu>::default());
                    continue;
                }
                let window = battle_menu
                    .0
                    .clone()
                    .unwrap_or(window::menu::battle::Window);
                commands.trigger(ToggleWindow(window::Window::BattleMenu(window)));
            }
            #[cfg(feature = "debug")]
            (ButtonState::Released, KeyCode::F11) => {
                commands.trigger(ToggleShowTiles);
            }
            #[cfg(feature = "debug")]
            (ButtonState::Released, KeyCode::F12) => {
                let window = debug.0.clone().unwrap_or_default();
                commands.trigger(ToggleWindow(Window::BattleDebug(Box::new(window))));
            }
            _ => {}
        }
    }
}
