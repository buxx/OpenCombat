use battle_core::{audio::Sound, types::WindowPoint};
use ggez::{
    graphics::{Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBuilder, Rect},
    Context, GameResult,
};
use oc_core::morale::SideMorale;

use crate::ui::component::Component;

use super::{
    battle::BATTLE_BUTTON_WIDTH,
    builder::{MARGIN, RIGHT_BOX_WIDTH},
    event::HudEvent,
};

pub const MORALE_INDICATOR_HEIGHT: f32 = 16.;

pub struct MoraleIndicator {
    point: WindowPoint,
    a_morale: SideMorale,
    b_morale: SideMorale,
}

impl MoraleIndicator {
    pub fn new(point: WindowPoint, a_morale: SideMorale, b_morale: SideMorale) -> Self {
        Self {
            point,
            a_morale,
            b_morale,
        }
    }
}

impl Component<HudEvent> for MoraleIndicator {
    fn point(&self, _ctx: &Context) -> WindowPoint {
        self.point
    }

    fn width(&self, _ctx: &Context) -> f32 {
        RIGHT_BOX_WIDTH - BATTLE_BUTTON_WIDTH - MARGIN
    }

    fn height(&self, _ctx: &Context) -> f32 {
        MORALE_INDICATOR_HEIGHT
    }

    fn sprites(&self, _ctx: &Context, _: &WindowPoint) -> Vec<DrawParam> {
        vec![]
    }

    fn event(&self, _ctx: &Context) -> Option<HudEvent> {
        None
    }

    fn sound(&self, _ctx: &Context) -> Option<Sound> {
        None
    }

    fn draw(&self, ctx: &mut Context, _hovered: &WindowPoint, canvas: &mut Canvas) -> GameResult {
        let a_total_width = self.width(ctx) / 2.;
        let b_total_width = self.width(ctx) / 2.;
        let start_a = self.point(ctx).x + (a_total_width * (1. - self.a_morale.0));
        let end_a = self.point(ctx).x + (self.width(ctx) / 2.);
        let start_b = end_a;
        let end_b = start_b + (b_total_width * self.b_morale.0);

        let mut mesh_builder = MeshBuilder::new();
        mesh_builder.rectangle(
            DrawMode::Fill(FillOptions::default()),
            Rect::new(
                self.point.x,
                self.point.y,
                self.width(ctx),
                self.height(ctx),
            ),
            Color::BLACK,
        )?;
        mesh_builder.rectangle(
            DrawMode::Fill(FillOptions::default()),
            Rect::new(
                start_a,
                self.point(ctx).y,
                end_a - start_a,
                self.height(ctx),
            ),
            Color::new(0.5, self.a_morale.0, 0., 1.),
        )?;
        mesh_builder.rectangle(
            DrawMode::Fill(FillOptions::default()),
            Rect::new(
                start_b,
                self.point(ctx).y,
                end_b - start_b,
                self.height(ctx),
            ),
            Color::new(0.5, self.b_morale.0, 0., 1.),
        )?;

        canvas.draw(
            &Mesh::from_data(ctx, mesh_builder.build()),
            DrawParam::new(),
        );

        Ok(())
    }
}
