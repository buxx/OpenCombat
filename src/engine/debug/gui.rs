use ggez::{
    graphics::{Canvas, DrawParam},
    Context,
};
use ggez_egui::egui;
use glam::Vec2;

use crate::{engine::Engine, message::Message};

use super::egui_backend;

impl Engine {
    pub fn update_debug_gui(&mut self, ctx: &mut Context) {
        let messages = self.debug_gui(ctx);
        let side_effects = self.react(messages);
        self.react_side_effects(side_effects, ctx);
        self.local_state.remove_finished_physics();
    }

    pub fn debug_gui(&mut self, ctx: &mut Context) -> Vec<Message> {
        let egui_ctx = egui_backend(ctx).ctx();
        egui::Window::new("Debug").show(&egui_ctx, |ui| {
            if ui.button("print \"hello world\"").clicked() {
                println!("hello world");
            }
            if ui.button("quit").clicked() {
                ctx.request_quit();
            }
        });
        egui_backend(ctx).update(ctx);

        vec![]
    }

    pub fn draw_debug_gui(&mut self, ctx: &mut Context, canvas: &mut Canvas) {
        let egui_backend = egui_backend(ctx);
        canvas.draw(egui_backend, DrawParam::default().dest(Vec2::new(0., 0.)));
    }
}
