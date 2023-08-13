use battle_core::types::WindowPoint;
use ggez::{
    graphics::{Canvas, DrawParam, MeshBuilder},
    Context, GameResult,
};

use crate::{engine::state::GuiState, ui::component::Component};

use super::Hud;

pub struct HudPainter<'a> {
    gui_state: &'a GuiState,
    hud: &'a Hud,
}

impl<'a> HudPainter<'a> {
    pub fn new(hud: &'a Hud, gui_state: &'a GuiState) -> Self {
        Self { hud, gui_state }
    }

    pub fn sprites(&self, ctx: &Context) -> Vec<DrawParam> {
        let hovered = &self.gui_state.current_cursor_window_point();
        [
            self.hud.background().sprites(ctx, hovered),
            self.hud.squad_statuses().sprites(ctx, hovered),
            self.hud.squad_detail().sprites(ctx, hovered),
            self.hud.battle_button().sprites(ctx, hovered),
            self.hud.morale_indicator().sprites(ctx, hovered),
            self.hud.minimap().sprites(ctx, hovered),
        ]
        .concat()
    }

    pub fn meshes(&self, _ctx: &Context, _mesh_builder: &mut MeshBuilder) -> GameResult<()> {
        Ok(())
    }

    pub fn draw(&self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        self.hud
            .squad_statuses()
            .draw(ctx, self.hover_point(), canvas)?;
        self.hud
            .squad_detail()
            .draw(ctx, self.hover_point(), canvas)?;
        self.hud
            .battle_button()
            .draw(ctx, self.hover_point(), canvas)?;
        self.hud
            .morale_indicator()
            .draw(ctx, self.hover_point(), canvas)?;
        self.hud.minimap().draw(ctx, self.hover_point(), canvas)?;

        Ok(())
    }

    fn hover_point(&self) -> &WindowPoint {
        self.gui_state.current_cursor_window_point()
    }
}
