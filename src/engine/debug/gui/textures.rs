use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Ui};

use crate::{engine::Engine, message::Message};

impl Engine {
    pub fn debug_gui_textures(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        vec![]
    }
}
