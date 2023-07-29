use battle_core::{audio::Sound, types::WindowPoint};
use ggez::{
    graphics::{Canvas, DrawParam},
    Context, GameResult,
};
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

    fn contains(&self, points: &Vec<&WindowPoint>) -> bool {
        points.iter().all(|point| {
            point.x >= self.point().x
                && point.x <= self.point().x + self.width()
                && point.y >= self.point().y
                && point.y <= self.point().y + self.height()
        })
    }
    fn event(&self) -> Option<E> {
        None
    }
    fn sound(&self) -> Option<Sound> {
        None
    }
    fn sprites(&self, _hovered: &WindowPoint) -> Vec<DrawParam> {
        vec![]
    }
    fn draw(&self, _ctx: &mut Context, _hovered: &WindowPoint, _canvas: &mut Canvas) -> GameResult {
        Ok(())
    }
}
