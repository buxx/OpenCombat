use bevy::{app::AppExit, prelude::Commands};
use bevy_egui::EguiContexts;

#[derive(Clone, Default)]
pub struct Window {
    /// Server sent the battle summary: battle end is persisted, application can exit
    pub resumed: bool,
}

impl Window {
    pub fn show(
        &mut self,
        contexts: &mut EguiContexts,
        commands: &mut Commands,
    ) -> bevy::prelude::Result {
        let ctx = contexts.ctx_mut()?;
        bevy_egui::egui::Window::new("BattleEnd")
            .pivot(bevy_egui::egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.label("Battle is end");
                if !self.resumed {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Saving battle ...");
                    });
                }
                if ui
                    .add_enabled(self.resumed, bevy_egui::egui::Button::new("Close"))
                    .clicked()
                {
                    commands.write_message(AppExit::Success);
                }
            });

        Ok(())
    }
}
