use ggez::{
    graphics::{Canvas, DrawParam},
    Context,
};
use ggez_egui::egui;
use glam::Vec2;

use crate::{
    engine::Engine,
    message::{LocalStateMessage, Message},
};

use super::egui_backend;

const EGUI_SCALE: f32 = 1.5;

pub mod body;
pub mod config;
pub mod header;
pub mod meta;
pub mod soldiers;
pub mod state;
pub mod terrain;

#[derive(PartialEq, Eq)]
pub enum Panel {
    Terrain,
    Soldiers,
    SharedState,
    LocalState,
    GlobalConfig,
    TerrainConfig,
    FightConfig,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Terrain
    }
}

impl Engine {
    pub fn update_debug_gui(&mut self, ctx: &mut Context) {
        let messages = self.debug_gui(ctx);
        let side_effects = self.react(messages);
        self.react_side_effects(side_effects, ctx);
    }

    pub fn debug_gui(&mut self, ctx: &mut Context) -> Vec<Message> {
        if !self.local_state.display_debug_gui() {
            return vec![Message::LocalState(LocalStateMessage::SetDebugGuiHovered(
                false,
            ))];
        }

        let drawable_size = ctx.gfx.drawable_size();
        egui_backend(ctx)
            .input
            .set_scale_factor(EGUI_SCALE, drawable_size);
        let egui_ctx = egui_backend(ctx).ctx();
        let mut messages = vec![];

        egui::Window::new("Debug").show(&egui_ctx, |ui| {
            messages.extend(self.debug_gui_header(ctx, &egui_ctx, ui));
            messages.extend(self.debug_gui_body(ctx, &egui_ctx, ui));
        });

        messages.push(Message::LocalState(LocalStateMessage::SetDebugGuiHovered(
            egui_ctx.is_pointer_over_area(),
        )));

        // FIXME BS NOW : If debug window not displayed, SetDebugGuiHovered(false)
        egui_backend(ctx).update(ctx);
        messages
    }

    pub fn draw_debug_gui(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        let egui_backend = egui_backend(ctx);
        canvas.draw(egui_backend, DrawParam::default().dest(Vec2::new(0., 0.)));
    }
}
