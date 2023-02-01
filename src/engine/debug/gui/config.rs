use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Grid, Slider, Ui};

use crate::{
    config::{
        FEELING_DECREASING_FREQ, INTERIORS_UPDATE_FREQ, SOLDIER_ANIMATE_FREQ, SOLDIER_UPDATE_FREQ,
        TARGET_FPS, VISIBILITY_UPDATE_FREQ,
    },
    engine::Engine,
    message::Message,
};

impl Engine {
    pub fn debug_gui_global_config(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        Grid::new("meta")
            .num_columns(3)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for (name, value, min, max, default) in [
                    (
                        "TARGET_FPS",
                        &mut self.config.target_fps,
                        1,
                        1500,
                        TARGET_FPS,
                    ),
                    (
                        "SOLDIER_UPDATE_FREQ",
                        &mut self.config.soldier_update_freq,
                        1,
                        120,
                        SOLDIER_UPDATE_FREQ,
                    ),
                    (
                        "SOLDIER_ANIMATE_FREQ",
                        &mut self.config.soldier_animate_freq,
                        1,
                        120,
                        SOLDIER_ANIMATE_FREQ,
                    ),
                    (
                        "INTERIORS_UPDATE_FREQ",
                        &mut self.config.interiors_update_freq,
                        1,
                        120,
                        INTERIORS_UPDATE_FREQ,
                    ),
                    (
                        "VISIBILITY_UPDATE_FREQ",
                        &mut self.config.visibility_update_freq,
                        1,
                        120,
                        VISIBILITY_UPDATE_FREQ,
                    ),
                    (
                        "FEELING_DECREASING_FREQ",
                        &mut self.config.feeling_decreasing_freq,
                        1,
                        120,
                        FEELING_DECREASING_FREQ,
                    ),
                ] {
                    ui.label(name);
                    if ui.button("reset").clicked() {
                        *value = default;
                    };
                    ui.add(Slider::new(value, min..=max));
                    ui.end_row();
                }
            });

        vec![]
    }

    pub fn debug_gui_terrain_config(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {});

        vec![]
    }

    pub fn debug_gui_fight_config(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {});

        vec![]
    }
}
