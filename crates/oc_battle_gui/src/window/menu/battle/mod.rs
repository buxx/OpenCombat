use bevy::prelude::*;
use bevy_egui::EguiContexts;
use oc_mod::Mod;
use oc_root::WorldConfig;

use crate::window::UnmountedWindow;

/// Used to cache debug battle window when not displayed
#[derive(Resource, Deref, DerefMut, Default)]
pub struct BattleMenuWindow(pub Option<Window>);

#[derive(Clone, Default)]
pub struct Window;

impl Window {
    pub fn show(
        &mut self,
        contexts: &mut EguiContexts,
        _commands: &mut Commands,
        _mod_: &Mod,
        _wcfg: &WorldConfig,
    ) -> Result {
        let ctx = contexts.ctx_mut()?;
        bevy_egui::egui::Window::new("Hello")
            .pivot(bevy_egui::egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.label("world");
            });

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct BattleMenuWindowPlugin;

impl Plugin for BattleMenuWindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BattleMenuWindow>()
            .add_observer(on_unmounted_window);
    }
}

fn on_unmounted_window(unmounted: On<UnmountedWindow>, mut window: ResMut<BattleMenuWindow>) {
    // Store unmounted debug window to reuse it later when want to display it again
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let crate::window::Window::BattleMenu(window_) = &unmounted.0 {
        window.0 = Some(window_.clone())
    }
}
