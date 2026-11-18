use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};
use oc_mod::Mod;
use oc_root::WorldConfig;
use oc_utils::let_some;

use crate::{
    states::{self, PointerIn},
    window::{battle::fight::FightWindowPlugin, menu::battle::BattleMenuWindowPlugin},
};

pub mod battle;
#[cfg(feature = "debug")]
pub mod debug;
pub mod menu;

// TODO: There is a lot of common code for windows, use generic
#[derive(Clone)]
pub enum Window {
    BattleMenu(menu::battle::Window),
    Fight(battle::fight::Window),
    BattleEnd(battle::end::Window),
    #[cfg(feature = "debug")]
    BattleDebug(Box<debug::battle::window::Window>),
}

impl Window {
    fn show(
        &mut self,
        contexts: &mut EguiContexts,
        commands: &mut Commands,
        mod_: &Mod,
        w: &WorldConfig,
        #[cfg_attr(not(feature = "debug"), allow(unused))] world: &crate::world::World,
        #[cfg_attr(not(feature = "debug"), allow(unused))] ingame: &crate::ingame::state::State,
    ) -> Result {
        match self {
            Window::BattleMenu(window) => window.show(contexts, commands, mod_, w)?,
            Window::Fight(window) => window.show(contexts, commands, mod_, w)?,
            Window::BattleEnd(window) => window.show(contexts, commands)?,
            #[cfg(feature = "debug")]
            Window::BattleDebug(window) => {
                window.show(contexts, commands, mod_, w, world, ingame)?
            }
        }

        Ok(())
    }
}

#[derive(Event)]
pub struct ToggleWindow(pub Window);

#[derive(Event)]
pub struct MountedWindow(#[allow(dead_code)] pub Window);

#[derive(Event)]
pub struct UnmountedWindow(pub Window);

pub fn on_toggle_debug_window(
    toggle: On<ToggleWindow>,
    mut commands: Commands,
    mut window: ResMut<crate::states::Window>,
    mut pointer: ResMut<NextState<PointerIn>>,
) {
    if let Some(window_) = &window.0 {
        if std::mem::discriminant(window_) == std::mem::discriminant(&toggle.0) {
            commands.trigger(UnmountedWindow(window_.clone()));
            window.0 = None;
        }
    } else {
        window.0 = Some(toggle.0.clone());
        commands.trigger(MountedWindow(toggle.0.clone()));
    }

    *pointer = NextState::Pending(PointerIn::Battle)
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BattleMenuWindowPlugin)
            .add_plugins(FightWindowPlugin)
            .add_systems(EguiPrimaryContextPass, show)
            .add_observer(on_toggle_debug_window);

        #[cfg(feature = "debug")]
        {
            app.add_plugins(debug::battle::DebugBattleWindowPlugin)
                .add_systems(
                    EguiPrimaryContextPass,
                    debug::end_battle::show
                        .run_if(in_state(crate::states::AppState::InGame))
                        .run_if(not(in_state(crate::ingame::BattlePhase::End))),
                );
        }
    }
}

fn show(
    mut contexts: EguiContexts,
    mut window: ResMut<states::Window>,
    mut commands: Commands,
    g: Res<states::GameConfig>,
    world: Res<crate::world::World>,
    ingame: Res<crate::ingame::state::State>,
) -> Result {
    let_some!(window = &mut window.0, return Ok(()));
    let_some!(g = &g.0, return Ok(()));

    window.show(&mut contexts, &mut commands, &g.mod_, &g.w, &world, &ingame)?;

    Ok(())
}
