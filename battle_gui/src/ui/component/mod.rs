use battle_core::types::WindowPoint;
use ggez::graphics::DrawParam;

pub mod background;

pub trait Component {
    fn sprites(&self) -> &Vec<DrawParam>;
    fn point(&self) -> &WindowPoint;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn contains(&self, point: &WindowPoint) -> bool;
}
