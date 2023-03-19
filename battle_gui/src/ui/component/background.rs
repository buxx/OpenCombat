use battle_core::types::{Scale, WindowPoint};
use ggez::graphics::{DrawParam, Rect};
use glam::Vec2;

pub struct HorizontalBackground {
    pub rel_left_start_x: f32,
    pub rel_left_start_y: f32,
    pub rel_left_width: f32,
    pub rel_left_height: f32,
    pub left_width: f32,
    pub left_height: f32,
    pub rel_center_start_x: f32,
    pub rel_center_start_y: f32,
    pub rel_center_width: f32,
    pub rel_center_height: f32,
    pub center_width: f32,
    pub center_height: f32,
    pub rel_right_start_x: f32,
    pub rel_right_start_y: f32,
    pub rel_right_width: f32,
    pub rel_right_height: f32,
    pub right_width: f32,
    pub right_height: f32,
}

impl HorizontalBackground {
    pub fn sprites(
        &self,
        point: WindowPoint,
        width: f32,
        height: f32,
        factor: f32,
    ) -> Vec<DrawParam> {
        vec![
            // Left part
            DrawParam::new()
                .src(Rect::new(
                    self.rel_left_start_x,
                    self.rel_left_start_y,
                    self.rel_left_width,
                    self.rel_left_height,
                ))
                .scale(Scale::new(factor, height / self.left_height).to_vec2())
                .dest(point.to_vec2()),
            // Center part
            DrawParam::new()
                .src(Rect::new(
                    self.rel_right_start_x,
                    self.rel_right_start_y,
                    self.rel_right_width,
                    self.rel_right_height,
                ))
                .scale(
                    Scale::new(
                        (width - self.left_width - self.right_width) / self.center_width,
                        height / self.center_height,
                    )
                    .to_vec2(),
                )
                .dest(point.to_vec2() + Vec2::new(self.left_width * factor, 0.)),
            // Right part
            DrawParam::new()
                .src(Rect::new(
                    self.rel_right_start_x,
                    self.rel_right_start_y,
                    self.rel_right_width,
                    self.rel_right_height,
                ))
                .scale(Scale::new(factor, height / self.right_height).to_vec2())
                .dest(point.to_vec2() + Vec2::new(width - self.right_width * factor, 0.)),
        ]
    }
}
