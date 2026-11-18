use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use oc_network::ToServer;

use crate::network::output::ToServerEvent;

pub fn show(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;
    egui::Window::new("Debug battle")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .resizable(false)
        .show(ctx, |ui| {
            if ui.button("End battle").clicked() {
                commands.trigger(ToServerEvent(ToServer::DebugEndBattle));
            }
        });

    Ok(())
}
