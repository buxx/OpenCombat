use battle_core::{audio::Sound, types::WindowPoint};
use ggez::graphics::{Canvas, DrawParam};
use glam::Vec2;

pub mod background;
pub mod button;

pub trait Component<E> {
    fn point(&self) -> WindowPoint;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn bounds(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }
    fn contains(&self, points: &Vec<&WindowPoint>) -> bool;
    fn event(&self) -> Option<E> {
        None
    }
    fn sound(&self) -> Option<Sound> {
        None
    }
    fn sprites(&self, _hovered: &WindowPoint) -> Vec<DrawParam> {
        vec![]
    }
    fn draw(&self, _hovered: &WindowPoint, _canvas: &mut Canvas) {}
}
