use battle_core::types::WindowPoint;

use crate::ui::component::Component;

use super::{
    builder::{MARGIN, RIGHT_BOX_WIDTH},
    event::HudEvent,
    morale::MORALE_INDICATOR_HEIGHT,
    HUD_HEIGHT,
};

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
        RIGHT_BOX_WIDTH - MARGIN * 2.0
    }

    fn height(&self, _ctx: &ggez::Context) -> f32 {
        HUD_HEIGHT - MORALE_INDICATOR_HEIGHT - MARGIN - MARGIN * 2.0
    }
}
