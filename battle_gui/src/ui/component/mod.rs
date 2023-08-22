use battle_core::{audio::Sound, types::WindowPoint};
use ggez::{
    graphics::{Canvas, DrawParam},
    Context, GameResult,
};
use glam::Vec2;

pub mod background;
pub mod button;

pub trait Component<E> {
    fn point(&self, ctx: &Context) -> WindowPoint;
    fn width(&self, ctx: &Context) -> f32;
    fn height(&self, ctx: &Context) -> f32;

    fn center(&self, ctx: &Context) -> WindowPoint {
        WindowPoint::new(
            self.point(ctx).x + self.width(ctx) / 2.,
            self.point(ctx).y + self.height(ctx) / 2.,
        )
    }
    fn bounds(&self, ctx: &Context) -> Vec2 {
        Vec2::new(self.width(ctx), self.height(ctx))
    }

    fn contains(&self, ctx: &Context, points: &[&WindowPoint]) -> bool {
        points.iter().all(|point| {
            point.x >= self.point(ctx).x
                && point.x <= self.point(ctx).x + self.width(ctx)
                && point.y >= self.point(ctx).y
                && point.y <= self.point(ctx).y + self.height(ctx)
        })
    }
    fn event(&self, _ctx: &Context) -> Option<E> {
        None
    }
    fn sound(&self, _ctx: &Context) -> Option<Sound> {
        None
    }
    fn sprites(&self, _ctx: &Context, _hovered: &WindowPoint) -> Vec<DrawParam> {
        vec![]
    }
    fn draw(&self, _ctx: &mut Context, _hovered: &WindowPoint, _canvas: &mut Canvas) -> GameResult {
        Ok(())
    }
}
