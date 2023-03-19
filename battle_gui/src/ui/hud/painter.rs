use ggez::{
    graphics::{Canvas, DrawParam, MeshBuilder},
    Context, GameResult,
};

use crate::ui::component::Component;

use super::Hud;

pub struct HudPainter<'a> {
    hud: &'a Hud,
}

impl<'a> HudPainter<'a> {
    pub fn new(hud: &'a Hud) -> Self {
        Self { hud }
    }

    pub fn sprites(&self) -> &Vec<DrawParam> {
        self.hud.background().sprites()
    }

    pub fn meshes(&self, _ctx: &Context, _mesh_builder: &mut MeshBuilder) -> GameResult<()> {
        Ok(())
    }

    pub fn draw(&self, _ctx: &mut Context, _canvas: &mut Canvas) {
        //
    }
}
