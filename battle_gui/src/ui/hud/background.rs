use battle_core::types::WindowPoint;
use ggez::graphics::DrawParam;

use crate::ui::component::Component;

pub struct Background {
    sprites: Vec<DrawParam>,
    point: WindowPoint,
    width: f32,
    height: f32,
}

impl Background {
    pub fn new(sprites: Vec<DrawParam>, point: WindowPoint, width: f32, height: f32) -> Self {
        Self {
            sprites,
            point,
            width,
            height,
        }
    }
}

impl Component for Background {
    fn sprites(&self) -> &Vec<DrawParam> {
        &self.sprites
    }

    fn point(&self) -> &WindowPoint {
        &self.point
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
    }

    fn contains(&self, point: &WindowPoint) -> bool {
        point.x >= self.point.x
            && point.x <= self.point.x + self.width
            && point.y >= self.point.y
            && point.y <= self.point.y + self.height
    }
}
