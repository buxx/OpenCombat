use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Grid, Ui};

use crate::{debug::DebugTerrain, engine::Engine, message::Message};

impl Engine {
    pub fn debug_gui_terrain(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        Grid::new("terrain_draw")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Draw");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut self.local_state.debug_terrain,
                        DebugTerrain::None,
                        "Normal",
                    );
                    ui.radio_value(
                        &mut self.local_state.debug_terrain,
                        DebugTerrain::Tiles,
                        "Tiles",
                    );
                    ui.radio_value(
                        &mut self.local_state.debug_terrain,
                        DebugTerrain::Opacity,
                        "Opacity",
                    );
                });
                ui.end_row();
            });

        vec![]
    }
}
