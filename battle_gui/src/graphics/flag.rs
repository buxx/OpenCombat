use battle_core::game::flag::FlagOwnership;
use oc_core::graphics::{FLAGS_SPRITE_SHEET_HEIGHT, FLAGS_SPRITE_SHEET_WIDTH};

use crate::utils::IntoSprite;

pub const FLAG_BLUE_START_X: f32 = 0.;
pub const FLAG_BLUE_START_Y: f32 = 0.;
pub const FLAG_BLUE_WIDTH: f32 = 50.;
pub const FLAG_BLUE_HEIGHT: f32 = 25.;

pub const FLAG_RED_START_X: f32 = 0.;
pub const FLAG_RED_START_Y: f32 = 25.;
pub const FLAG_RED_WIDTH: f32 = 50.;
pub const FLAG_RED_HEIGHT: f32 = 25.;

pub const FLAG_BOTH_START_X: f32 = 0.;
pub const FLAG_BOTH_START_Y: f32 = 50.;
pub const FLAG_BOTH_WIDTH: f32 = 50.;
pub const FLAG_BOTH_HEIGHT: f32 = 25.;

pub const FLAG_GRAY_START_X: f32 = 0.;
pub const FLAG_GRAY_START_Y: f32 = 75.;
pub const FLAG_GRAY_WIDTH: f32 = 50.;
pub const FLAG_GRAY_HEIGHT: f32 = 25.;

pub const FLAG_REL_BLUE_START_X: f32 = FLAG_BLUE_START_X / FLAGS_SPRITE_SHEET_WIDTH;
pub const FLAG_REL_BLUE_START_Y: f32 = FLAG_BLUE_START_Y / FLAGS_SPRITE_SHEET_HEIGHT;
pub const FLAG_REL_BLUE_WIDTH: f32 = FLAG_BLUE_WIDTH / FLAGS_SPRITE_SHEET_WIDTH;
pub const FLAG_REL_BLUE_HEIGHT: f32 = FLAG_BLUE_HEIGHT / FLAGS_SPRITE_SHEET_HEIGHT;

pub const FLAG_REL_RED_START_X: f32 = FLAG_RED_START_X / FLAGS_SPRITE_SHEET_WIDTH;
pub const FLAG_REL_RED_START_Y: f32 = FLAG_RED_START_Y / FLAGS_SPRITE_SHEET_HEIGHT;
pub const FLAG_REL_RED_WIDTH: f32 = FLAG_RED_WIDTH / FLAGS_SPRITE_SHEET_WIDTH;
pub const FLAG_REL_RED_HEIGHT: f32 = FLAG_RED_HEIGHT / FLAGS_SPRITE_SHEET_HEIGHT;

pub const FLAG_REL_BOTH_START_X: f32 = FLAG_BOTH_START_X / FLAGS_SPRITE_SHEET_WIDTH;
pub const FLAG_REL_BOTH_START_Y: f32 = FLAG_BOTH_START_Y / FLAGS_SPRITE_SHEET_HEIGHT;
pub const FLAG_REL_BOTH_WIDTH: f32 = FLAG_BOTH_WIDTH / FLAGS_SPRITE_SHEET_WIDTH;
pub const FLAG_REL_BOTH_HEIGHT: f32 = FLAG_BOTH_HEIGHT / FLAGS_SPRITE_SHEET_HEIGHT;

pub const FLAG_REL_GRAY_START_X: f32 = FLAG_GRAY_START_X / FLAGS_SPRITE_SHEET_WIDTH;
pub const FLAG_REL_GRAY_START_Y: f32 = FLAG_GRAY_START_Y / FLAGS_SPRITE_SHEET_HEIGHT;
pub const FLAG_REL_GRAY_WIDTH: f32 = FLAG_GRAY_WIDTH / FLAGS_SPRITE_SHEET_WIDTH;
pub const FLAG_REL_GRAY_HEIGHT: f32 = FLAG_GRAY_HEIGHT / FLAGS_SPRITE_SHEET_HEIGHT;

impl IntoSprite for FlagOwnership {
    fn to_relative_array(&self) -> [f32; 4] {
        match self {
            FlagOwnership::A => [
                FLAG_REL_BLUE_START_X,
                FLAG_REL_BLUE_START_Y,
                FLAG_REL_BLUE_WIDTH,
                FLAG_REL_BLUE_HEIGHT,
            ],
            FlagOwnership::B => [
                FLAG_REL_RED_START_X,
                FLAG_REL_RED_START_Y,
                FLAG_REL_RED_WIDTH,
                FLAG_REL_RED_HEIGHT,
            ],
            FlagOwnership::Both => [
                FLAG_REL_BOTH_START_X,
                FLAG_REL_BOTH_START_Y,
                FLAG_REL_BOTH_WIDTH,
                FLAG_REL_BOTH_HEIGHT,
            ],
            FlagOwnership::Nobody => [
                FLAG_REL_GRAY_START_X,
                FLAG_REL_GRAY_START_Y,
                FLAG_REL_GRAY_WIDTH,
                FLAG_REL_GRAY_HEIGHT,
            ],
        }
    }
}
