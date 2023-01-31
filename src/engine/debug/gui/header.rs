use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Ui};

use crate::{
    engine::Engine,
    message::{LocalStateMessage, Message},
};

impl Engine {
    pub fn debug_gui_header(
        &mut self,
        _ctx: &mut Context,
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

            let cursor_text = format!("Cursor {}", self.local_state.get_debug_physics());
            if ui.button(&cursor_text).clicked() {
                let new_debug_physics = self.local_state.get_debug_physics().next();
                messages.extend(vec![
                    Message::LocalState(LocalStateMessage::SetControl(new_debug_physics.control())),
                    Message::LocalState(LocalStateMessage::SetDebugPhysics(new_debug_physics)),
                ]);
            }

            ui.separator();

            ui.checkbox(&mut self.local_state.debug_mouse, "Cursor");
            ui.checkbox(&mut self.local_state.debug_move_paths, "Move");
            ui.checkbox(&mut self.local_state.debug_formation_positions, "Formation");
            ui.checkbox(&mut self.local_state.debug_scene_item_circles, "Soldier");
            ui.checkbox(&mut self.local_state.debug_areas, "Areas");
            ui.checkbox(&mut self.local_state.debug_visibilities, "Visibilities");
        });

        ui.separator();

        messages
    }
}
