use battle_core::types::WindowPoint;
use ggez::graphics::{DrawParam, Rect};

pub struct Button {
    pub rel_start_x: f32,
    pub rel_start_y: f32,
    pub rel_width: f32,
    pub rel_height: f32,
}

impl Button {
    pub fn sprites(&self, point: WindowPoint, enabled: bool, hovered: bool) -> Vec<DrawParam> {
        let rel_start_x = if enabled {
            if hovered {
                self.rel_start_x + self.rel_width * 1.
            } else {
                self.rel_start_x
            }
        } else {
            self.rel_start_x + self.rel_width * 2.
        };

        vec![DrawParam::new()
            .src(Rect::new(
                rel_start_x,
                self.rel_start_y,
                self.rel_width,
                self.rel_height,
            ))
            .dest(point.to_vec2())]
    }
}
