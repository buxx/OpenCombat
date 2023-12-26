use ggegui::egui::{Context as EguiContext, Grid, Slider, Ui};
use ggez::Context;

use battle_core::config::{
    ChangeConfigMessage, FEELING_DECREASING_FREQ, INTERIORS_UPDATE_FREQ, SOLDIER_ANIMATE_FREQ,
    SOLDIER_UPDATE_FREQ, TARGET_FPS, TILE_TYPE_OPACITY_BRICK_WALL, TILE_TYPE_OPACITY_CONCRETE,
    TILE_TYPE_OPACITY_DEEP_WATER, TILE_TYPE_OPACITY_DIRT, TILE_TYPE_OPACITY_HEDGE,
    TILE_TYPE_OPACITY_HIGH_GRASS, TILE_TYPE_OPACITY_LIGHT_UNDERBRUSH,
    TILE_TYPE_OPACITY_MIDDLE_GRASS, TILE_TYPE_OPACITY_MIDDLE_ROCK,
    TILE_TYPE_OPACITY_MIDDLE_WOOD_LOGS, TILE_TYPE_OPACITY_MUD, TILE_TYPE_OPACITY_SHORT_GRASS,
    TILE_TYPE_OPACITY_TRUNK, TILE_TYPE_OPACITY_UNDERBRUSH, TILE_TYPE_OPACITY_WATER,
    VISIBILITY_BY_LAST_FRAME_SHOOT, VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE,
    VISIBILITY_DEAD_MODIFIER, VISIBILITY_DEFEND_MODIFIER, VISIBILITY_ENGAGE_MODIFIER,
    VISIBILITY_FIRSTS, VISIBILITY_HIDE_MODIFIER, VISIBILITY_IDLE_CROUCH_MODIFIER,
    VISIBILITY_IDLE_LYING_MODIFIER, VISIBILITY_IDLE_STANDUP_MODIFIER,
    VISIBILITY_IN_VEHICLE_MODIFIER, VISIBILITY_MOVE_FAST_TO_MODIFIER, VISIBILITY_MOVE_TO_MODIFIER,
    VISIBILITY_SNEAK_TO_MODIFIER, VISIBILITY_SUPPRESS_FIRE_MODIFIER,
    VISIBILITY_UNCONSCIOUS_MODIFIER, VISIBILITY_UPDATE_FREQ, VISIBLE_STARTS_AT,
};

use crate::{
    engine::{message::EngineMessage, Engine},
    graphics::message::GraphicsMessage,
};

impl Engine {
    pub fn debug_gui_server_config(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        Grid::new("meta")
            .num_columns(3)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                for (name, value, min, max, default, message) in [
                    (
                        "SOLDIER_UPDATE_FREQ",
                        &mut self.server_config.soldier_update_freq,
                        1,
                        120,
                        SOLDIER_UPDATE_FREQ,
                        ChangeConfigMessage::SoldierUpdateFreq,
                    ),
                    (
                        "SOLDIER_ANIMATE_FREQ",
                        &mut self.server_config.soldier_animate_freq,
                        1,
                        120,
                        SOLDIER_ANIMATE_FREQ,
                        ChangeConfigMessage::SoldierAnimateFreq,
                    ),
                    (
                        "INTERIORS_UPDATE_FREQ",
                        &mut self.server_config.interiors_update_freq,
                        1,
                        120,
                        INTERIORS_UPDATE_FREQ,
                        ChangeConfigMessage::InteriorsUpdateFreq,
                    ),
                    (
                        "VISIBILITY_UPDATE_FREQ",
                        &mut self.server_config.visibility_update_freq,
                        1,
                        120,
                        VISIBILITY_UPDATE_FREQ,
                        ChangeConfigMessage::VisibilityUpdateFreq,
                    ),
                    (
                        "FEELING_DECREASING_FREQ",
                        &mut self.server_config.feeling_decreasing_freq,
                        1,
                        120,
                        FEELING_DECREASING_FREQ,
                        ChangeConfigMessage::FeelingDecreasingFreq,
                    ),
                ]
                    as [(_, _, _, _, _, fn(_) -> _); 5]
                {
                    ui.label(name);
                    if ui.button("reset").clicked() {
                        *value = default;
                    };
                    if ui.add(Slider::new(value, min..=max)).changed() {
                        messages.push(EngineMessage::ChangeServerConfig(message(*value)));
                    };
                    ui.end_row();
                }
            });

        messages
    }
    pub fn debug_gui_gui_config(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        Grid::new("meta")
            .num_columns(3)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("TARGET_FPS");
                if ui.button("reset").clicked() {
                    self.config.target_fps = TARGET_FPS as u32;
                };
                ui.add(Slider::new(&mut self.config.target_fps, 1..=3000));
                ui.end_row();

                ui.label("INTERIORS_UPDATE_FREQ");
                if ui.button("reset").clicked() {
                    self.config.interiors_update_freq = INTERIORS_UPDATE_FREQ;
                };
                ui.add(Slider::new(&mut self.config.interiors_update_freq, 0..=300));
                ui.end_row();
            });

        vec![]
    }

    pub fn debug_gui_visibility_config(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("VISIBILITY_FIRSTS");
                if ui.button("reset").clicked() {
                    self.server_config.visibility_firsts = VISIBILITY_FIRSTS;
                };
                if ui
                    .add(Slider::new(
                        &mut self.server_config.visibility_firsts,
                        0..=10,
                    ))
                    .changed()
                {
                    messages.push(EngineMessage::ChangeServerConfig(
                        ChangeConfigMessage::VisibilityFirsts(self.server_config.visibility_firsts),
                    ));
                };
                ui.end_row();

                ui.label("VISIBLE_STARTS_AT");
                if ui.button("reset").clicked() {
                    self.server_config.visible_starts_at = VISIBLE_STARTS_AT;
                };
                if ui
                    .add(Slider::new(
                        &mut self.server_config.visible_starts_at,
                        (0.)..=1.,
                    ))
                    .changed()
                {
                    messages.push(EngineMessage::ChangeServerConfig(
                        ChangeConfigMessage::VisibleStartsAt(self.server_config.visible_starts_at),
                    ));
                };
                ui.end_row();

                for (name, value, default, message) in [
                    (
                        "IDLE STANDUP",
                        &mut self.server_config.visibility_idle_standup_modifier,
                        VISIBILITY_IDLE_STANDUP_MODIFIER,
                        ChangeConfigMessage::VisibilityIdleStandupModifier,
                    ),
                    (
                        "IDLE CROUCHED",
                        &mut self.server_config.visibility_idle_crouch_modifier,
                        VISIBILITY_IDLE_CROUCH_MODIFIER,
                        ChangeConfigMessage::VisibilityIdleCrouchModifier,
                    ),
                    (
                        "IDLE LYING",
                        &mut self.server_config.visibility_idle_lying_modifier,
                        VISIBILITY_IDLE_LYING_MODIFIER,
                        ChangeConfigMessage::VisibilityIdleLyingModifier,
                    ),
                    (
                        "MOVE_TO",
                        &mut self.server_config.visibility_move_to_modifier,
                        VISIBILITY_MOVE_TO_MODIFIER,
                        ChangeConfigMessage::VisibilityMoveModifier,
                    ),
                    (
                        "MOVE_FAST_TO",
                        &mut self.server_config.visibility_move_fast_to_modifier,
                        VISIBILITY_MOVE_FAST_TO_MODIFIER,
                        ChangeConfigMessage::VisibilityMoveFastModifier,
                    ),
                    (
                        "SNEAK_TO",
                        &mut self.server_config.visibility_sneak_to_modifier,
                        VISIBILITY_SNEAK_TO_MODIFIER,
                        ChangeConfigMessage::VisibilitySneakToModifier,
                    ),
                    (
                        "DEFEND",
                        &mut self.server_config.visibility_defend_modifier,
                        VISIBILITY_DEFEND_MODIFIER,
                        ChangeConfigMessage::VisibilityDefendModifier,
                    ),
                    (
                        "HIDE",
                        &mut self.server_config.visibility_hide_modifier,
                        VISIBILITY_HIDE_MODIFIER,
                        ChangeConfigMessage::VisibilityHideModifier,
                    ),
                    (
                        "IN_VEHICLE",
                        &mut self.server_config.visibility_in_vehicle_modifier,
                        VISIBILITY_IN_VEHICLE_MODIFIER,
                        ChangeConfigMessage::VisibilityInVehicleModifier,
                    ),
                    (
                        "SUPPRESS_FIRE",
                        &mut self.server_config.visibility_suppress_fire_modifier,
                        VISIBILITY_SUPPRESS_FIRE_MODIFIER,
                        ChangeConfigMessage::VisibilitySuppressFireModifier,
                    ),
                    (
                        "ENGAGE",
                        &mut self.server_config.visibility_engage_modifier,
                        VISIBILITY_ENGAGE_MODIFIER,
                        ChangeConfigMessage::VisibilityEngageModifier,
                    ),
                    (
                        "DEAD",
                        &mut self.server_config.visibility_dead_modifier,
                        VISIBILITY_DEAD_MODIFIER,
                        ChangeConfigMessage::VisibilityDeadModifier,
                    ),
                    (
                        "UNCONSCIOUS",
                        &mut self.server_config.visibility_unconscious_modifier,
                        VISIBILITY_UNCONSCIOUS_MODIFIER,
                        ChangeConfigMessage::VisibilityUnconsciousModifier,
                    ),
                ]
                    as [(_, _, _, fn(_) -> _); 13]
                {
                    ui.label(format!("VISIBILITY_BEHAVIOR_MODIFIER__{}", name));
                    if ui.button("reset").clicked() {
                        *value = default;
                    };
                    if ui.add(Slider::new(value, (-10.)..=10.)).changed() {
                        messages.push(EngineMessage::ChangeServerConfig(message(*value)));
                    };
                    ui.end_row();
                }

                for (name, value, default, message) in [
                    (
                        "SHORT_GRASS",
                        &mut self.server_config.tile_type_opacity_short_grass,
                        TILE_TYPE_OPACITY_SHORT_GRASS,
                        ChangeConfigMessage::TileTypeOpacityShortGrass,
                    ),
                    (
                        "MIDDLE_GRASS",
                        &mut self.server_config.tile_type_opacity_middle_grass,
                        TILE_TYPE_OPACITY_MIDDLE_GRASS,
                        ChangeConfigMessage::TileTypeOpacityMiddleGrass,
                    ),
                    (
                        "HIGH_GRASS",
                        &mut self.server_config.tile_type_opacity_high_grass,
                        TILE_TYPE_OPACITY_HIGH_GRASS,
                        ChangeConfigMessage::TileTypeOpacityHighGrass,
                    ),
                    (
                        "DIRT",
                        &mut self.server_config.tile_type_opacity_dirt,
                        TILE_TYPE_OPACITY_DIRT,
                        ChangeConfigMessage::TileTypeOpacityDirt,
                    ),
                    (
                        "CONCRETE",
                        &mut self.server_config.tile_type_opacity_concrete,
                        TILE_TYPE_OPACITY_CONCRETE,
                        ChangeConfigMessage::TileTypeOpacityConcrete,
                    ),
                    (
                        "MUD",
                        &mut self.server_config.tile_type_opacity_mud,
                        TILE_TYPE_OPACITY_MUD,
                        ChangeConfigMessage::TileTypeOpacityMud,
                    ),
                    (
                        "WALL",
                        &mut self.server_config.tile_type_opacity_brick_wall,
                        TILE_TYPE_OPACITY_BRICK_WALL,
                        ChangeConfigMessage::TileTypeOpacityBrickWall,
                    ),
                    (
                        "TRUNK",
                        &mut self.server_config.tile_type_opacity_trunk,
                        TILE_TYPE_OPACITY_TRUNK,
                        ChangeConfigMessage::TileTypeOpacityTrunk,
                    ),
                    (
                        "WATER",
                        &mut self.server_config.tile_type_opacity_water,
                        TILE_TYPE_OPACITY_WATER,
                        ChangeConfigMessage::TileTypeOpacityWater,
                    ),
                    (
                        "DEEP_WATER",
                        &mut self.server_config.tile_type_opacity_deep_water,
                        TILE_TYPE_OPACITY_DEEP_WATER,
                        ChangeConfigMessage::TileTypeOpacityDeepWater,
                    ),
                    (
                        "UNDERBRUSH",
                        &mut self.server_config.tile_type_opacity_underbrush,
                        TILE_TYPE_OPACITY_UNDERBRUSH,
                        ChangeConfigMessage::TileTypeOpacityUnderbrush,
                    ),
                    (
                        "LIGHT_UNDERBRUSH",
                        &mut self.server_config.tile_type_opacity_light_underbrush,
                        TILE_TYPE_OPACITY_LIGHT_UNDERBRUSH,
                        ChangeConfigMessage::TileTypeOpacityLightUnderbrush,
                    ),
                    (
                        "MIDDLE_WOOD_LOGS",
                        &mut self.server_config.tile_type_opacity_middle_wood_logs,
                        TILE_TYPE_OPACITY_MIDDLE_WOOD_LOGS,
                        ChangeConfigMessage::TileTypeOpacityMiddleWoodLogs,
                    ),
                    (
                        "HEDGE",
                        &mut self.server_config.tile_type_opacity_hedge,
                        TILE_TYPE_OPACITY_HEDGE,
                        ChangeConfigMessage::TileTypeOpacityHedge,
                    ),
                    (
                        "MIDDLE_ROCK",
                        &mut self.server_config.tile_type_opacity_middle_rock,
                        TILE_TYPE_OPACITY_MIDDLE_ROCK,
                        ChangeConfigMessage::TileTypeOpacityMiddleRock,
                    ),
                ]
                    as [(_, _, _, fn(_) -> _); 15]
                {
                    ui.label(format!("TILE_TYPE_OPACITY_{}", name));
                    if ui.button("reset").clicked() {
                        *value = default;
                    };
                    if ui.add(Slider::new(value, (0.)..=1.)).changed() {
                        messages.push(EngineMessage::Graphics(
                            GraphicsMessage::RecomputeDebugTerrainOpacity,
                        ));
                        messages.push(EngineMessage::ChangeServerConfig(message(*value)))
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
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        Grid::new("meta")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("VISIBILITY_BY_LAST_FRAME_SHOOT");
                if ui.button("reset").clicked() {
                    self.server_config.visibility_by_last_frame_shoot =
                        VISIBILITY_BY_LAST_FRAME_SHOOT;
                }
                if ui
                    .add(Slider::new(
                        &mut self.server_config.visibility_by_last_frame_shoot,
                        0..=600,
                    ))
                    .changed()
                {
                    messages.push(EngineMessage::ChangeServerConfig(
                        ChangeConfigMessage::VisibilityByLastFrameShot(
                            self.server_config.visibility_by_last_frame_shoot,
                        ),
                    ));
                };
                ui.end_row();

                ui.label("VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE");
                if ui.button("reset").clicked() {
                    self.server_config.visibility_by_last_frame_shoot_distance =
                        VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE;
                }
                if ui
                    .add(Slider::new(
                        &mut self.server_config.visibility_by_last_frame_shoot_distance,
                        0..=30,
                    ))
                    .changed()
                {
                    messages.push(EngineMessage::ChangeServerConfig(
                        ChangeConfigMessage::VisibilityByLastFrameShotDistance(
                            self.server_config.visibility_by_last_frame_shoot_distance,
                        ),
                    ));
                }
            });

        messages
    }
}
