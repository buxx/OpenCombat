use battle_core::game::squad::CurrentAction;
use oc_core::graphics::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH};

use crate::utils::IntoSprite;

pub const ACTION_EMPTY_START_X: f32 = 300.;
pub const ACTION_EMPTY_START_Y: f32 = 0.;
pub const ACTION_WALKING_START_X: f32 = 246.;
pub const ACTION_WALKING_START_Y: f32 = 12.;
pub const ACTION_RUNNING_START_X: f32 = 276.;
pub const ACTION_RUNNING_START_Y: f32 = 12.;
pub const ACTION_CRAWLING_START_X: f32 = 288.;
pub const ACTION_CRAWLING_START_Y: f32 = 12.;
pub const ACTION_TARGET_FIRING_START_X: f32 = 300.;
pub const ACTION_TARGET_FIRING_START_Y: f32 = 12.;
pub const ACTION_SUPPRESS_FIRING_START_X: f32 = 312.;
pub const ACTION_SUPPRESS_FIRING_START_Y: f32 = 12.;
pub const ACTION_AIMING_START_X: f32 = 264.;
pub const ACTION_AIMING_START_Y: f32 = 24.;
pub const ACTION_RELOADING_START_X: f32 = 276.;
pub const ACTION_RELOADING_START_Y: f32 = 24.;
pub const ACTION_DEFENDING_START_X: f32 = 288.;
pub const ACTION_DEFENDING_START_Y: f32 = 24.;
pub const ACTION_HIDING_START_X: f32 = 300.;
pub const ACTION_HIDING_START_Y: f32 = 24.;
pub const ACTION_WIDTH: f32 = 12.;
pub const ACTION_HEIGHT: f32 = 12.;

pub const ACTION_EMPTY_START_REL_X: f32 = ACTION_EMPTY_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_EMPTY_START_REL_Y: f32 = ACTION_EMPTY_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_WALKING_START_REL_X: f32 = ACTION_WALKING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_WALKING_START_REL_Y: f32 = ACTION_WALKING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_RUNNING_START_REL_X: f32 = ACTION_RUNNING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_RUNNING_START_REL_Y: f32 = ACTION_RUNNING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_CRAWLING_START_REL_X: f32 = ACTION_CRAWLING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_CRAWLING_START_REL_Y: f32 = ACTION_CRAWLING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_TARGET_FIRING_START_REL_X: f32 =
    ACTION_TARGET_FIRING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_TARGET_FIRING_START_REL_Y: f32 =
    ACTION_TARGET_FIRING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_SUPPRESS_FIRING_START_REL_X: f32 =
    ACTION_SUPPRESS_FIRING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_SUPPRESS_FIRING_START_REL_Y: f32 =
    ACTION_SUPPRESS_FIRING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_AIMING_START_REL_X: f32 = ACTION_AIMING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_AIMING_START_REL_Y: f32 = ACTION_AIMING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_RELOADING_START_REL_X: f32 = ACTION_RELOADING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_RELOADING_START_REL_Y: f32 = ACTION_RELOADING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_DEFENDING_START_REL_X: f32 = ACTION_DEFENDING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_DEFENDING_START_REL_Y: f32 = ACTION_DEFENDING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_HIDING_START_REL_X: f32 = ACTION_HIDING_START_X / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_HIDING_START_REL_Y: f32 = ACTION_HIDING_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const ACTION_REL_WIDTH: f32 = ACTION_WIDTH / UI_SPRITE_SHEET_WIDTH;
pub const ACTION_REL_HEIGHT: f32 = ACTION_HEIGHT / UI_SPRITE_SHEET_HEIGHT;

impl IntoSprite for CurrentAction {
    fn to_relative_array(&self) -> [f32; 4] {
        match self {
            CurrentAction::Idle => [
                ACTION_EMPTY_START_REL_X,
                ACTION_EMPTY_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Walking => [
                ACTION_WALKING_START_REL_X,
                ACTION_WALKING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Running => [
                ACTION_RUNNING_START_REL_X,
                ACTION_RUNNING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Crawling => [
                ACTION_CRAWLING_START_REL_X,
                ACTION_CRAWLING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::TargetFiring => [
                ACTION_TARGET_FIRING_START_REL_X,
                ACTION_TARGET_FIRING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::SuppressFiring => [
                ACTION_SUPPRESS_FIRING_START_REL_X,
                ACTION_SUPPRESS_FIRING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Aiming => [
                ACTION_AIMING_START_REL_X,
                ACTION_AIMING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Reloading => [
                ACTION_RELOADING_START_REL_X,
                ACTION_RELOADING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Defending => [
                ACTION_DEFENDING_START_REL_X,
                ACTION_DEFENDING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Hiding => [
                ACTION_HIDING_START_REL_X,
                ACTION_HIDING_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Driving => [
                ACTION_EMPTY_START_REL_X,
                ACTION_EMPTY_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
            CurrentAction::Rotating => [
                ACTION_EMPTY_START_REL_X,
                ACTION_EMPTY_START_REL_Y,
                ACTION_REL_WIDTH,
                ACTION_REL_HEIGHT,
            ],
        }
    }
}
