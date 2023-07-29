use ggez::{
    graphics::{Canvas, DrawParam},
    Context,
};
use glam::Vec2;

use super::Engine;

pub const EGUI_SCALE: f32 = 1.5;

impl Engine {
    pub fn draw_egui(&mut self, _ctx: &mut Context, canvas: &mut Canvas) {
        canvas.draw(
            &self.egui_backend,
            DrawParam::default().dest(Vec2::new(0., 0.)),
        );
    }
}
