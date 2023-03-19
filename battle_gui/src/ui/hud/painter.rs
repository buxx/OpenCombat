use battle_core::{state::battle::BattleState, types::WindowPoint};
use ggez::{
    graphics::{Canvas, Color, DrawMode, DrawParam, MeshBuilder, Rect, Text},
    Context, GameResult,
};
use glam::Vec2;

use crate::engine::state::GuiState;

const HUD_HEIGHT: f32 = 200.0;

pub struct HudPainter<'a> {
    gui_state: &'a GuiState,
    battle_state: &'a BattleState,
}

impl<'a> HudPainter<'a> {
    pub fn new(gui_state: &'a GuiState, battle_state: &'a BattleState) -> Self {
        Self {
            gui_state,
            battle_state,
        }
    }

    pub fn contains(&self, ctx: &Context, point: &WindowPoint) -> bool {
        let (start, end) = (self.start(ctx), self.end(ctx));
        point.x >= start.x && point.x <= end.x && point.y >= start.y && point.y <= end.y
    }

    pub fn sprites(&self) -> Vec<DrawParam> {
        vec![]
    }

    pub fn meshes(&self, ctx: &Context, mesh_builder: &mut MeshBuilder) -> GameResult<()> {
        mesh_builder.rectangle(DrawMode::fill(), self.area(ctx), Color::GREEN)?;

        Ok(())
    }

    pub fn draw(&self, ctx: &Context, canvas: &mut Canvas) {
        let window = ctx.gfx.window().inner_size();
        if self.battle_state.phase().placement() && !self.battle_state.ready(self.gui_state.side())
        {
            let start = self.start(ctx);
            let dest_point = Vec2::new(
                start.x + (window.width as f32 / 2.0) - 320.0,
                start.y + HUD_HEIGHT / 2.0,
            );
            canvas.draw(
                Text::new("Click here when ready to battle").set_scale(48.),
                dest_point,
            );
        }
    }

    fn start(&self, ctx: &Context) -> WindowPoint {
        let window = ctx.gfx.window().inner_size();
        WindowPoint::new(0., window.height as f32 - HUD_HEIGHT)
    }

    fn end(&self, ctx: &Context) -> WindowPoint {
        let window = ctx.gfx.window().inner_size();
        WindowPoint::new(window.width as f32, window.height as f32)
    }

    fn area(&self, ctx: &Context) -> Rect {
        let window = ctx.gfx.window().inner_size();
        Rect::new(
            0.,
            window.height as f32 - HUD_HEIGHT,
            window.width as f32,
            window.height as f32,
        )
    }
}
