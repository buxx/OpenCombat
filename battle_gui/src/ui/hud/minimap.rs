use battle_core::types::WindowPoint;

use crate::ui::component::Component;

use super::{
    battle::BATTLE_BUTTON_HEIGHT,
    builder::{MARGIN, RIGHT_BOX_WIDTH},
    event::HudEvent,
    HUD_HEIGHT,
};

pub const MINIMAP_WIDTH: f32 = RIGHT_BOX_WIDTH - MARGIN;
pub const MINIMAP_HEIGHT: f32 = HUD_HEIGHT - BATTLE_BUTTON_HEIGHT - (MARGIN * 2.);

pub struct Minimap {
    point: WindowPoint,
}

impl Minimap {
    pub fn new(point: WindowPoint) -> Self {
        Self { point }
    }
}

impl Component<HudEvent> for Minimap {
    fn point(&self, _ctx: &ggez::Context) -> WindowPoint {
        self.point
    }

    fn width(&self, _ctx: &ggez::Context) -> f32 {
        MINIMAP_WIDTH
    }

    fn height(&self, _ctx: &ggez::Context) -> f32 {
        MINIMAP_HEIGHT
    }
}
