use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Grid, Ui};

use crate::{engine::Engine, message::Message};

impl Engine {
    pub fn debug_gui_shared_state(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for (name, value) in self.shared_state.debug_lines() {
                    ui.label(name);
                    ui.label(value);
                    ui.end_row();
                }
            });

        vec![]
    }

    pub fn debug_gui_local_state(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for (name, value) in self.local_state.debug_lines() {
                    ui.label(&name);
                    ui.label(&value);
                    ui.end_row();
                }
            });

        vec![]
    }
}
