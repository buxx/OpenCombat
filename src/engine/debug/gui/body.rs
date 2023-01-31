use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Ui};

use crate::{engine::Engine, message::Message};

use super::Panel;

impl Engine {
    pub fn debug_gui_body(
        &mut self,
        ctx: &mut Context,
        egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        let mut messages = vec![];

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.debug_gui.panel, Panel::Terrain, "Terrain");
            ui.selectable_value(&mut self.debug_gui.panel, Panel::Soldiers, "Soldiers");
        });
        ui.separator();

        match self.debug_gui.panel {
            Panel::Terrain => {
                self.debug_gui_terrain(ctx, egui_ctx, ui);
            }
            Panel::Soldiers => {
                self.debug_gui_soldiers(ctx, egui_ctx, ui);
            }
        }

        messages
    }
}
