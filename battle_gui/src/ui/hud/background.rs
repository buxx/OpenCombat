use battle_core::{
    config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH},
    types::WindowPoint,
};
use ggez::graphics::DrawParam;

use crate::ui::component::{background::HorizontalBackground, Component};

use super::event::HudEvent;

pub const BACKGROUND_LEFT_START_X: f32 = 0.;
pub const BACKGROUND_LEFT_START_Y: f32 = 250.;
pub const BACKGROUND_LEFT_WIDTH: f32 = 20.;
pub const BACKGROUND_LEFT_HEIGHT: f32 = 200.;

pub const BACKGROUND_CENTER_START_X: f32 = 20.;
pub const BACKGROUND_CENTER_START_Y: f32 = 250.;
pub const BACKGROUND_CENTER_WIDTH: f32 = 10.;
pub const BACKGROUND_CENTER_HEIGHT: f32 = 200.;

pub const BACKGROUND_RIGHT_START_X: f32 = 30.;
pub const BACKGROUND_RIGHT_START_Y: f32 = 250.;
pub const BACKGROUND_RIGHT_WIDTH: f32 = 20.;
pub const BACKGROUND_RIGHT_HEIGHT: f32 = 200.;

pub const BACKGROUND_REL_LEFT_START_X: f32 = BACKGROUND_LEFT_START_X / UI_SPRITE_SHEET_WIDTH;
pub const BACKGROUND_REL_LEFT_START_Y: f32 = BACKGROUND_LEFT_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const BACKGROUND_REL_LEFT_WIDTH: f32 = BACKGROUND_LEFT_WIDTH / UI_SPRITE_SHEET_WIDTH;
pub const BACKGROUND_REL_LEFT_HEIGHT: f32 = BACKGROUND_LEFT_HEIGHT / UI_SPRITE_SHEET_HEIGHT;

pub const BACKGROUND_REL_CENTER_START_X: f32 = BACKGROUND_CENTER_START_X / UI_SPRITE_SHEET_WIDTH;
pub const BACKGROUND_REL_CENTER_START_Y: f32 = BACKGROUND_CENTER_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const BACKGROUND_REL_CENTER_WIDTH: f32 = BACKGROUND_CENTER_WIDTH / UI_SPRITE_SHEET_WIDTH;
pub const BACKGROUND_REL_CENTER_HEIGHT: f32 = BACKGROUND_CENTER_HEIGHT / UI_SPRITE_SHEET_HEIGHT;

pub const BACKGROUND_REL_RIGHT_START_X: f32 = BACKGROUND_RIGHT_START_X / UI_SPRITE_SHEET_WIDTH;
pub const BACKGROUND_REL_RIGHT_START_Y: f32 = BACKGROUND_RIGHT_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const BACKGROUND_REL_RIGHT_WIDTH: f32 = BACKGROUND_RIGHT_WIDTH / UI_SPRITE_SHEET_WIDTH;
pub const BACKGROUND_REL_RIGHT_HEIGHT: f32 = BACKGROUND_RIGHT_HEIGHT / UI_SPRITE_SHEET_HEIGHT;

pub struct Background {
    point: WindowPoint,
    width: f32,
    height: f32,
}

impl Background {
    pub fn new(point: WindowPoint, width: f32, height: f32) -> Self {
        Self {
            point,
            width,
            height,
        }
    }
}

impl Component<HudEvent> for Background {
    fn point(&self) -> WindowPoint {
        self.point.clone()
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        self.height
    }

    fn contains(&self, points: &Vec<&WindowPoint>) -> bool {
        points.iter().all(|point| {
            point.x >= self.point.x
                && point.x <= self.point.x + self.width
                && point.y >= self.point.y
                && point.y <= self.point.y + self.height
        })
    }

    fn sprites(&self, _hovered: &WindowPoint) -> Vec<DrawParam> {
        HorizontalBackground {
            rel_left_start_x: BACKGROUND_REL_LEFT_START_X,
            rel_left_start_y: BACKGROUND_REL_LEFT_START_Y,
            rel_left_width: BACKGROUND_REL_LEFT_WIDTH,
            rel_left_height: BACKGROUND_REL_LEFT_HEIGHT,
            left_width: BACKGROUND_LEFT_WIDTH,
            left_height: BACKGROUND_LEFT_HEIGHT,
            rel_center_start_x: BACKGROUND_REL_CENTER_START_X,
            rel_center_start_y: BACKGROUND_REL_CENTER_START_Y,
            rel_center_width: BACKGROUND_REL_CENTER_WIDTH,
            rel_center_height: BACKGROUND_REL_CENTER_HEIGHT,
            center_width: BACKGROUND_CENTER_WIDTH,
            center_height: BACKGROUND_CENTER_HEIGHT,
            rel_right_start_x: BACKGROUND_REL_RIGHT_START_X,
            rel_right_start_y: BACKGROUND_REL_RIGHT_START_Y,
            rel_right_width: BACKGROUND_REL_RIGHT_WIDTH,
            rel_right_height: BACKGROUND_REL_RIGHT_HEIGHT,
            right_width: BACKGROUND_RIGHT_WIDTH,
            right_height: BACKGROUND_RIGHT_HEIGHT,
        }
        .sprites(self.point, self.width, self.height)
    }
}
