use battle_core::{
    config::ChangeConfigMessage,
    state::battle::{message::BattleStateMessage, phase::Phase},
};
use ggegui::egui::{Context as EguiContext, Ui};
use ggez::Context;

use crate::{
    debug::DebugPhysics,
    engine::{
        message::{EngineMessage, GuiStateMessage},
        Engine,
    },
};
use strum::IntoEnumIterator;

impl Engine {
    pub fn debug_gui_header(
        &mut self,
        ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        ui.horizontal(|ui| {
            let side_text = format!("Side {}", self.gui_state.side());
            if ui.button(&side_text).clicked() {
                messages.push(EngineMessage::GuiState(GuiStateMessage::ChangeSide))
            }

            ui.separator();

            ui.checkbox(&mut self.gui_state.debug_mouse, "Cursor");
            ui.checkbox(&mut self.gui_state.debug_move_paths, "Move");
            if ui
                .checkbox(&mut self.gui_state.debug_formation_positions, "Formation")
                .changed()
            {
                messages.push(EngineMessage::ChangeServerConfig(
                    ChangeConfigMessage::SendDebugPoints(self.gui_state.debug_formation_positions),
                ));
            };
            ui.checkbox(&mut self.gui_state.debug_scene_item_circles, "Soldier");
            ui.checkbox(&mut self.gui_state.debug_areas, "Areas");
            ui.checkbox(&mut self.gui_state.debug_visibilities, "Visibilities");
            ui.checkbox(&mut self.gui_state.debug_targets, "Targets");
            ui.checkbox(&mut self.gui_state.debug_physics_areas, "Physics");

            ui.label(format!("FPS : {:.2}", ctx.time.fps()));
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Phase");
            for phase in Phase::iter() {
                let text = phase.to_string();
                if ui
                    .radio_value(self.battle_state.phase_mut(), phase.clone(), text)
                    .changed()
                {
                    messages.push(EngineMessage::BattleState(BattleStateMessage::SetPhase(
                        phase.clone(),
                    )));
                }
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Cursor physics");
            ui.horizontal(|ui| {
                let changes = vec![
                    ui.radio_value(self.gui_state.debug_physics_mut(), DebugPhysics::None, "No")
                        .changed(),
                    ui.radio_value(
                        self.gui_state.debug_physics_mut(),
                        DebugPhysics::MosinNagantM1924GunFire,
                        "MosinNagantM1924",
                    )
                    .changed(),
                    ui.radio_value(
                        self.gui_state.debug_physics_mut(),
                        DebugPhysics::BrandtMle2731Shelling,
                        "BrandtMle2731",
                    )
                    .changed(),
                ];

                if changes.iter().any(|v| *v) {
                    messages.extend(vec![EngineMessage::GuiState(GuiStateMessage::SetControl(
                        self.physics_control(self.gui_state.get_debug_physics()),
                    ))]);
                }
            });
        });

        ui.separator();

        messages
    }
}
