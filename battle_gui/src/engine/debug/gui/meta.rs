use ggegui::egui::{Context as EguiContext, Grid, Ui};
use ggez::Context;

use crate::{engine::message::EngineMessage, engine::Engine};

impl Engine {
    pub fn debug_gui_shared_state(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for (name, value) in self.battle_state.debug_lines() {
                    ui.label(name);
                    ui.label(value);
                    ui.end_row();
                }
            });

        vec![]
    }

    pub fn debug_gui_gui_state(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for (name, value) in self.gui_state.debug_lines() {
                    ui.label(&name);
                    ui.label(&value);
                    ui.end_row();
                }
            });

        vec![]
    }
}
