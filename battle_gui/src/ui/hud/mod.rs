use self::background::Background;
use battle_core::{
    config::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH},
    types::WindowPoint,
};

use super::component::Component;

pub mod background;
pub mod builder;
pub mod painter;

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

pub const HUD_HEIGHT: f32 = 200.0;

pub struct Hud {
    background: Background,
}

impl Hud {
    pub fn new(background: Background) -> Self {
        Self { background }
    }

    pub fn background(&self) -> &Background {
        &self.background
    }

    pub fn contains(&self, point: &WindowPoint) -> bool {
        self.background.contains(point)
    }
}
