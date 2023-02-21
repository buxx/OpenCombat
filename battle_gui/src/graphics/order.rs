use battle_core::{
    order::marker::OrderMarker,
    types::{Angle, WindowPoint, WorldPoint},
};
use ggez::graphics::{DrawParam, Rect};

use super::Graphics;

impl Graphics {
    pub fn order_marker_draw_params(
        &self,
        order_marker: &OrderMarker,
        draw_to: WindowPoint,
        angle: Angle,
    ) -> DrawParam {
        let sprite_info = order_marker.sprite_info();
        DrawParam::new()
            .src(Rect::new(
                sprite_info.relative_start_x,
                sprite_info.relative_start_y,
                sprite_info.relative_width,
                sprite_info.relative_height,
            ))
            .dest(draw_to.to_vec2())
            .rotation(angle.0)
            .offset(sprite_info.offset().to_vec2())
    }

    pub fn order_marker_selection_rect(
        &self,
        order_marker: &OrderMarker,
        from: WorldPoint,
    ) -> Rect {
        let sprite_info = order_marker.sprite_info();
        Rect::new(
            from.x - sprite_info.half_width,
            from.y - sprite_info.half_height,
            sprite_info.width,
            sprite_info.height,
        )
    }
}
