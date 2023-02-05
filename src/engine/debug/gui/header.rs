use ggegui::egui::{Context as EguiContext, Ui};
use ggez::Context;

use crate::{
    debug::DebugPhysics,
    engine::Engine,
    message::{LocalStateMessage, Message},
};

impl Engine {
    pub fn debug_gui_header(
        &mut self,
        ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        let mut messages = vec![];

        ui.horizontal(|ui| {
            let side_text = format!("Side {}", self.local_state.side());
            if ui.button(&side_text).clicked() {
                messages.push(Message::LocalState(LocalStateMessage::ChangeSide))
            }

            ui.separator();

            ui.checkbox(&mut self.local_state.debug_mouse, "Cursor");
            ui.checkbox(&mut self.local_state.debug_move_paths, "Move");
            ui.checkbox(&mut self.local_state.debug_formation_positions, "Formation");
            ui.checkbox(&mut self.local_state.debug_scene_item_circles, "Soldier");
            ui.checkbox(&mut self.local_state.debug_areas, "Areas");
            ui.checkbox(&mut self.local_state.debug_visibilities, "Visibilities");

            ui.label(format!("FPS : {:.2}", ctx.time.fps()));
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Cursor physics");
            ui.horizontal(|ui| {
                let changes = vec![
                    ui.radio_value(
                        self.local_state.debug_physics_mut(),
                        DebugPhysics::None,
                        "No",
                    )
                    .changed(),
                    ui.radio_value(
                        self.local_state.debug_physics_mut(),
                        DebugPhysics::MosinNagantM1924GunFire,
                        "MosinNagantM1924",
                    )
                    .changed(),
                    ui.radio_value(
                        self.local_state.debug_physics_mut(),
                        DebugPhysics::BrandtMle2731Shelling,
                        "BrandtMle2731",
                    )
                    .changed(),
                ];

                if changes.iter().any(|v| *v) {
                    messages.extend(vec![Message::LocalState(LocalStateMessage::SetControl(
                        self.local_state.get_debug_physics().control(),
                    ))]);
                }
            });
        });

        ui.separator();

        messages
    }
}
