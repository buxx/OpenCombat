use battle_core::types::{WindowPoint, WorldPoint};
use ggez::Context;

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
    map_width: f32,
    map_height: f32,
}

impl Minimap {
    pub fn new(point: WindowPoint, map_width: f32, map_height: f32) -> Self {
        Self {
            point,
            map_width,
            map_height,
        }
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

    fn event(&self, ctx: &Context) -> Option<HudEvent> {
        let mouse_position = ctx.mouse.position();
        let in_x = mouse_position.x - self.point(ctx).x;
        let in_y = mouse_position.y - self.point(ctx).y;
        let relative_x = in_x / self.width(ctx);
        let relative_y = in_y / self.height(ctx);

        let world_point =
            WorldPoint::new(self.map_width * relative_x, self.map_height * relative_y);

        Some(HudEvent::CenterMapOn(world_point))
    }
}
