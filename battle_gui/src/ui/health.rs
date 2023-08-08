use oc_core::{
    graphics::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH},
    health::Health,
};

use crate::utils::{IntoSprite, GREEN, RED, YELLOW};

use super::color::Colorized;

pub const HEALTH_OK_START_X: f32 = 228.;
pub const HEALTH_OK_START_Y: f32 = 0.;
// pub const HEALTH_INJURED_START_X: f32 = 228.;
// pub const HEALTH_INJURED_START_Y: f32 = 12.;
pub const HEALTH_DEAD_START_X: f32 = 228.;
pub const HEALTH_DEAD_START_Y: f32 = 24.;
pub const HEALTH_WIDTH: f32 = 12.;
pub const HEALTH_HEIGHT: f32 = 12.;

pub const HEALTH_OK_REL_START_X: f32 = HEALTH_OK_START_X / UI_SPRITE_SHEET_WIDTH;
pub const HEALTH_OK_REL_START_Y: f32 = HEALTH_OK_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const HEALTH_DEAD_REL_START_X: f32 = HEALTH_DEAD_START_X / UI_SPRITE_SHEET_WIDTH;
pub const HEALTH_DEAD_REL_START_Y: f32 = HEALTH_DEAD_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const HEALTH_REL_WIDTH: f32 = HEALTH_WIDTH / UI_SPRITE_SHEET_WIDTH;
pub const HEALTH_REL_HEIGHT: f32 = HEALTH_HEIGHT / UI_SPRITE_SHEET_HEIGHT;

impl Colorized for Health {
    fn color(&self) -> ggez::graphics::Color {
        match self {
            Health::Good => GREEN,
            Health::Unconscious => YELLOW,
            Health::Dead => RED,
        }
    }
}

impl IntoSprite for Health {
    fn to_relative_array(&self) -> [f32; 4] {
        match self {
            Health::Good => [
                HEALTH_OK_REL_START_X,
                HEALTH_OK_REL_START_Y,
                HEALTH_REL_WIDTH,
                HEALTH_REL_HEIGHT,
            ],
            Health::Unconscious => [
                HEALTH_DEAD_REL_START_X,
                HEALTH_DEAD_REL_START_Y,
                HEALTH_REL_WIDTH,
                HEALTH_REL_HEIGHT,
            ],
            Health::Dead => [
                HEALTH_DEAD_REL_START_X,
                HEALTH_DEAD_REL_START_Y,
                HEALTH_REL_WIDTH,
                HEALTH_REL_HEIGHT,
            ],
        }
    }
}
