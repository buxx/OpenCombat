use crate::config::{
    DISPLAY_DEFEND_X_OFFSET, DISPLAY_DEFEND_Y_OFFSET, UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH,
};
use crate::types::*;
use crate::utils::rect_contains;
use ggez::graphics::{self, Rect};

use super::PendingOrder;

const ORDER_MARKER_START_X: f32 = 0.0;
const ORDER_MARKER_START_Y: f32 = 100.0;
const ORDER_MARKER_WIDTH: f32 = 11.0;
const ORDER_MARKER_HEIGHT: f32 = 11.0;
const ORDER_MARKER_DEFEND_START_Y: f32 = 150.0;
const ORDER_MARKER_DEFEND_WIDTH: f32 = 50.0;
const ORDER_MARKER_DEFEND_HEIGHT: f32 = 50.0;
const ORDER_MARKER_HIDE_START_Y: f32 = 200.0;
const ORDER_MARKER_HIDE_WIDTH: f32 = 50.0;
const ORDER_MARKER_HIDE_HEIGHT: f32 = 50.0;

#[derive(PartialEq, Clone, Debug)]
pub enum OrderMarker {
    MoveTo,
    MoveFastTo,
    SneakTo,
    FireTo,
    Defend,
    Hide,
}

impl OrderMarker {
    pub fn selectable(&self) -> Offset {
        match self {
            OrderMarker::MoveTo
            | OrderMarker::MoveFastTo
            | OrderMarker::SneakTo
            | OrderMarker::FireTo => Offset::new(1.0, 1.0),
            OrderMarker::Defend | OrderMarker::Hide => Offset::new(1.0, 0.33),
        }
    }

    pub fn sprite_info(&self) -> OrderMarkerSpriteInfo {
        match self {
            OrderMarker::MoveTo => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: ORDER_MARKER_START_Y / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
                half_width: ORDER_MARKER_WIDTH / 2.0,
                half_height: ORDER_MARKER_HEIGHT / 2.0,
            },
            OrderMarker::MoveFastTo => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 1.0))
                    / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
                half_width: ORDER_MARKER_WIDTH / 2.0,
                half_height: ORDER_MARKER_HEIGHT / 2.0,
            },
            OrderMarker::SneakTo => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 2.0))
                    / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
                half_width: ORDER_MARKER_WIDTH / 2.0,
                half_height: ORDER_MARKER_HEIGHT / 2.0,
            },
            OrderMarker::FireTo => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: (ORDER_MARKER_START_Y + (ORDER_MARKER_HEIGHT * 3.0))
                    / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_WIDTH,
                height: ORDER_MARKER_HEIGHT,
                half_width: ORDER_MARKER_WIDTH / 2.0,
                half_height: ORDER_MARKER_HEIGHT / 2.0,
            },
            OrderMarker::Defend => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: ORDER_MARKER_DEFEND_START_Y / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_DEFEND_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_DEFEND_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_DEFEND_WIDTH,
                height: ORDER_MARKER_DEFEND_HEIGHT,
                half_width: ORDER_MARKER_DEFEND_WIDTH / 2.0,
                half_height: ORDER_MARKER_DEFEND_HEIGHT / 2.0,
            },
            OrderMarker::Hide => OrderMarkerSpriteInfo {
                relative_start_x: ORDER_MARKER_START_X / UI_SPRITE_SHEET_WIDTH,
                relative_start_y: ORDER_MARKER_HIDE_START_Y / UI_SPRITE_SHEET_HEIGHT,
                relative_width: ORDER_MARKER_HIDE_WIDTH / UI_SPRITE_SHEET_WIDTH,
                relative_height: ORDER_MARKER_HIDE_HEIGHT / UI_SPRITE_SHEET_HEIGHT,
                width: ORDER_MARKER_HIDE_WIDTH,
                height: ORDER_MARKER_HIDE_HEIGHT,
                half_width: ORDER_MARKER_HIDE_WIDTH / 2.0,
                half_height: ORDER_MARKER_HIDE_HEIGHT / 2.0,
            },
        }
    }

    pub fn to_pending_order(&self) -> PendingOrder {
        match self {
            OrderMarker::MoveTo => PendingOrder::MoveTo,
            OrderMarker::MoveFastTo => PendingOrder::MoveFastTo,
            OrderMarker::SneakTo => PendingOrder::SneakTo,
            OrderMarker::FireTo => todo!(),
            OrderMarker::Defend => PendingOrder::Defend,
            OrderMarker::Hide => PendingOrder::Hide,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderMarkerSpriteInfo {
    pub relative_start_x: f32,
    pub relative_start_y: f32,
    pub relative_width: f32,
    pub relative_height: f32,
    pub width: f32,
    pub height: f32,
    pub half_width: f32,
    pub half_height: f32,
}

impl OrderMarkerSpriteInfo {
    pub fn as_draw_params(
        &self,
        draw_to: WindowPoint,
        angle: Angle,
        offset: Offset,
    ) -> graphics::DrawParam {
        graphics::DrawParam::new()
            .src(graphics::Rect::new(
                self.relative_start_x,
                self.relative_start_y,
                self.relative_width,
                self.relative_height,
            ))
            .dest(draw_to.to_vec2())
            .rotation(angle.0)
            .offset(offset.to_vec2())
    }

    pub fn rectangle(&self, from_point: &WindowPoint) -> graphics::Rect {
        graphics::Rect::new(
            from_point.x - self.half_width,
            from_point.y - self.half_height,
            self.width,
            self.height,
        )
    }

    pub fn selection_rect(&self, from: WorldPoint) -> Rect {
        Rect::new(
            from.x - self.half_width,
            from.y - self.half_height,
            self.width,
            self.height,
        )
    }

    // pub fn get_selection_rect2(&self, from: WorldPoint) -> Rect {
    //     Rect::new(from.x, from.y, self.width, self.height)
    // }

    pub fn contains(
        &self,
        draw_to: &WindowPoint,
        rotation: (Angle, Offset),
        cursor: &WindowPoint,
    ) -> bool {
        rect_contains(&self.rectangle(draw_to), rotation, cursor.to_vec2())
    }
}
