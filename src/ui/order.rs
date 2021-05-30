use crate::behavior::order::Order;
use crate::config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH};
use crate::{SceneItemId, ScenePoint};
use ggez::graphics;

const ORDER_MARKER_START_X: f32 = 0.0;
const ORDER_MARKER_START_Y: f32 = 100.0;
const ORDER_MARKER_WIDTH: f32 = 11.0;
const ORDER_MARKER_HEIGHT: f32 = 11.0;

#[derive(PartialEq, Clone, Debug)]
pub enum OrderMarker {
    MoveTo(SceneItemId, ScenePoint),
    MoveFastTo(SceneItemId, ScenePoint),
    HideTo(SceneItemId, ScenePoint),
    FireTo(SceneItemId, ScenePoint),
}

impl OrderMarker {
    pub fn new(scene_item_id: SceneItemId, order: &Order) -> Self {
        match order {
            Order::MoveTo(move_to_scene_point) => {
                OrderMarker::MoveTo(scene_item_id, *move_to_scene_point)
            }
            Order::MoveFastTo(move_to_scene_point) => {
                OrderMarker::MoveFastTo(scene_item_id, *move_to_scene_point)
            }
            Order::HideTo(move_to_scene_point) => {
                OrderMarker::HideTo(scene_item_id, *move_to_scene_point)
            }
        }
    }

    fn get_scene_point(&self) -> ScenePoint {
        match self {
            OrderMarker::MoveTo(_, scene_point)
            | OrderMarker::MoveFastTo(_, scene_point)
            | OrderMarker::HideTo(_, scene_point)
            | OrderMarker::FireTo(_, scene_point) => *scene_point,
        }
    }
    pub fn get_scene_item_id(&self) -> SceneItemId {
        match self {
            OrderMarker::MoveTo(scene_item_id, _)
            | OrderMarker::MoveFastTo(scene_item_id, _)
            | OrderMarker::HideTo(scene_item_id, _)
            | OrderMarker::FireTo(scene_item_id, _) => *scene_item_id,
        }
    }
    pub fn set_scene_point(&mut self, new_scene_point: ScenePoint) {
        match self {
            OrderMarker::MoveTo(_, scene_point)
            | OrderMarker::MoveFastTo(_, scene_point)
            | OrderMarker::HideTo(_, scene_point)
            | OrderMarker::FireTo(_, scene_point) => {
                // FIXME: When fire, re compute "on scene item" ?
                scene_point.x = new_scene_point.x;
                scene_point.y = new_scene_point.y
            }
        }
    }
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

    pub fn point_is_inside(&self, draw_to_scene_point: &ScenePoint, point: &ScenePoint) -> bool {
        let dest_scene_point = ScenePoint::new(
            draw_to_scene_point.x - (self.width / 2.0),
            draw_to_scene_point.y - (self.height / 2.0),
        );

        point.x >= dest_scene_point.x
            && point.x <= (dest_scene_point.x + self.width)
            && point.y >= draw_to_scene_point.y
            && point.y <= (draw_to_scene_point.y + self.height)
    }
}

impl OrderMarker {
    pub fn sprite_info(&self) -> OrderMarkerSpriteInfo {
        match self {
            OrderMarker::MoveTo(_, _) => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: ORDER_MARKER_START_Y / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
            },
            OrderMarker::MoveFastTo(_, _) => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 1.0))
                    / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
            },
            OrderMarker::HideTo(_, _) => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 2.0))
                    / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
            },
            OrderMarker::FireTo(_, _) => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 3.0))
                    / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
            },
        }
    }
}
