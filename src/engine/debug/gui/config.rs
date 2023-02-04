use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Grid, Slider, Ui};

use crate::{
    config::{
        FEELING_DECREASING_FREQ, INTERIORS_UPDATE_FREQ, SOLDIER_ANIMATE_FREQ, SOLDIER_UPDATE_FREQ,
        TARGET_FPS, TILE_TYPE_OPACITY_BRICK_WALL, TILE_TYPE_OPACITY_CONCRETE,
        TILE_TYPE_OPACITY_DIRT, TILE_TYPE_OPACITY_HIGH_GRASS, TILE_TYPE_OPACITY_MIDDLE_GRASS,
        TILE_TYPE_OPACITY_MUD, TILE_TYPE_OPACITY_SHORT_GRASS, VISIBILITY_BY_LAST_FRAME_SHOOT,
        VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE, VISIBILITY_DEAD_MODIFIER,
        VISIBILITY_DEFEND_MODIFIER, VISIBILITY_ENGAGE_MODIFIER, VISIBILITY_FIRSTS,
        VISIBILITY_HIDE_MODIFIER, VISIBILITY_IDLE_MODIFIER, VISIBILITY_IN_VEHICLE_MODIFIER,
        VISIBILITY_MOVE_FAST_TO_MODIFIER, VISIBILITY_MOVE_TO_MODIFIER,
        VISIBILITY_SNEAK_TO_MODIFIER, VISIBILITY_SUPPRESS_FIRE_MODIFIER,
        VISIBILITY_UNCONSCIOUS_MODIFIER, VISIBILITY_UPDATE_FREQ, VISIBLE_STARTS_AT,
    },
    engine::Engine,
    message::{GraphicsMessage, Message},
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

    pub fn debug_gui_visibility_config(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        let mut messages = vec![];

        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("VISIBILITY_FIRSTS");
                if ui.button("reset").clicked() {
                    *&mut self.config.visibility_firsts = VISIBILITY_FIRSTS;
                };
                ui.add(Slider::new(&mut self.config.visibility_firsts, 0..=10));
                ui.end_row();

                ui.label("VISIBLE_STARTS_AT");
                if ui.button("reset").clicked() {
                    *&mut self.config.visible_starts_at = VISIBLE_STARTS_AT;
                };
                ui.add(Slider::new(&mut self.config.visible_starts_at, (0.)..=1.));
                ui.end_row();

                for (name, value, default) in [
                    (
                        "IDLE",
                        &mut self.config.visibility_idle_modifier,
                        VISIBILITY_IDLE_MODIFIER,
                    ),
                    (
                        "MOVE_TO",
                        &mut self.config.visibility_move_to_modifier,
                        VISIBILITY_MOVE_TO_MODIFIER,
                    ),
                    (
                        "MOVE_FAST_TO",
                        &mut self.config.visibility_move_fast_to_modifier,
                        VISIBILITY_MOVE_FAST_TO_MODIFIER,
                    ),
                    (
                        "SNEAK_TO",
                        &mut self.config.visibility_sneak_to_modifier,
                        VISIBILITY_SNEAK_TO_MODIFIER,
                    ),
                    (
                        "DEFEND",
                        &mut self.config.visibility_defend_modifier,
                        VISIBILITY_DEFEND_MODIFIER,
                    ),
                    (
                        "HIDE",
                        &mut self.config.visibility_hide_modifier,
                        VISIBILITY_HIDE_MODIFIER,
                    ),
                    (
                        "IN_VEHICLE",
                        &mut self.config.visibility_in_vehicle_modifier,
                        VISIBILITY_IN_VEHICLE_MODIFIER,
                    ),
                    (
                        "SUPPRESS_FIRE",
                        &mut self.config.visibility_suppress_fire_modifier,
                        VISIBILITY_SUPPRESS_FIRE_MODIFIER,
                    ),
                    (
                        "ENGAGE",
                        &mut self.config.visibility_engage_modifier,
                        VISIBILITY_ENGAGE_MODIFIER,
                    ),
                    (
                        "DEAD",
                        &mut self.config.visibility_dead_modifier,
                        VISIBILITY_DEAD_MODIFIER,
                    ),
                    (
                        "UNCONSCIOUS",
                        &mut self.config.visibility_unconscious_modifier,
                        VISIBILITY_UNCONSCIOUS_MODIFIER,
                    ),
                ] {
                    ui.label(format!("VISIBILITY_BEHAVIOR_MODIFIER__{}", name));
                    if ui.button("reset").clicked() {
                        *value = default;
                    };
                    ui.add(Slider::new(value, (-5.)..=5.));
                    ui.end_row();
                }

                for (name, value, default) in [
                    (
                        "SHORT_GRASS",
                        &mut self.config.tile_type_opacity_short_grass,
                        TILE_TYPE_OPACITY_SHORT_GRASS,
                    ),
                    (
                        "MIDDLE_GRASS",
                        &mut self.config.tile_type_opacity_middle_grass,
                        TILE_TYPE_OPACITY_MIDDLE_GRASS,
                    ),
                    (
                        "HIGH_GRASS",
                        &mut self.config.tile_type_opacity_high_grass,
                        TILE_TYPE_OPACITY_HIGH_GRASS,
                    ),
                    (
                        "DIRT",
                        &mut self.config.tile_type_opacity_dirt,
                        TILE_TYPE_OPACITY_DIRT,
                    ),
                    (
                        "CONCRETE",
                        &mut self.config.tile_type_opacity_concrete,
                        TILE_TYPE_OPACITY_CONCRETE,
                    ),
                    (
                        "MUD",
                        &mut self.config.tile_type_opacity_mud,
                        TILE_TYPE_OPACITY_MUD,
                    ),
                    (
                        "WALL",
                        &mut self.config.tile_type_opacity_brick_wall,
                        TILE_TYPE_OPACITY_BRICK_WALL,
                    ),
                ] {
                    ui.label(format!("TILE_TYPE_OPACITY_{}", name));
                    if ui.button("reset").clicked() {
                        *value = default;
                    };
                    if ui.add(Slider::new(value, (0.)..=1.)).changed() {
                        messages.push(Message::Graphics(
                            GraphicsMessage::RecomputeDebugTerrainOpacity,
                        ))
                    };
                    ui.end_row();
                }
            });

        messages
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
            .show(ui, |ui| {
                ui.label("VISIBILITY_BY_LAST_FRAME_SHOOT");
                if ui.button("reset").clicked() {
                    self.config.visibility_by_last_frame_shoot = VISIBILITY_BY_LAST_FRAME_SHOOT;
                }
                ui.add(Slider::new(
                    &mut self.config.visibility_by_last_frame_shoot,
                    0..=600,
                ));
                ui.end_row();

                ui.label("VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE");
                if ui.button("reset").clicked() {
                    self.config.visibility_by_last_frame_shoot_distance =
                        VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE;
                }
                ui.add(Slider::new(
                    &mut self.config.visibility_by_last_frame_shoot_distance,
                    0..=30,
                ))
            });

        vec![]
    }
}
