use crate::config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH};
use crate::ScenePoint;
use ggez::graphics;

const ORDER_MARKER_START_X: f32 = 0.0;
const ORDER_MARKER_START_Y: f32 = 100.0;
const ORDER_MARKER_WIDTH: f32 = 11.0;
const ORDER_MARKER_HEIGHT: f32 = 11.0;

pub enum OrderMarker {
    MoveTo,
    MoveFastTo,
    HideTo,
    FireTo,
}

pub struct OrderMarkerSpriteInfo {
    pub relative_start_x: f32,
    pub relative_start_y: f32,
    pub relative_width: f32,
    pub relative_height: f32,
    pub width: f32,
    pub height: f32,
}

impl OrderMarkerSpriteInfo {
    pub fn as_draw_params(&self, draw_to_scene_point: &ScenePoint) -> graphics::DrawParam {
        let dest_scene_point = ScenePoint::new(
            draw_to_scene_point.x - (self.width / 2.0),
            draw_to_scene_point.y - (self.height / 2.0),
        );
        graphics::DrawParam::new()
            .src(graphics::Rect::new(
                self.relative_start_x,
                self.relative_start_y,
                self.relative_width,
                self.relative_height,
            ))
            .dest(dest_scene_point)
    }
}

impl OrderMarker {
    pub fn sprite_info(&self) -> OrderMarkerSpriteInfo {
        match self {
            OrderMarker::MoveTo => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: ORDER_MARKER_START_Y / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
            },
            OrderMarker::MoveFastTo => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 1.0)) / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
            },
            OrderMarker::HideTo => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 2.0)) / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
            },
            OrderMarker::FireTo => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 3.0)) / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
            },
        }
    }
}
