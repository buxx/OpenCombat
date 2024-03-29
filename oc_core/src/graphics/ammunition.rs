use super::{UI_SPRITE_SHEET_HEIGHT, UI_SPRITE_SHEET_WIDTH};

pub const AMMUNITION_RESERVE_STATUS_OK_START_X: f32 = 264.;
pub const AMMUNITION_RESERVE_STATUS_OK_START_Y: f32 = 0.;
pub const AMMUNITION_RESERVE_STATUS_LOW_START_X: f32 = 276.;
pub const AMMUNITION_RESERVE_STATUS_LOW_START_Y: f32 = 0.;
pub const AMMUNITION_RESERVE_STATUS_EMPTY_START_X: f32 = 288.;
pub const AMMUNITION_RESERVE_STATUS_EMPTY_START_Y: f32 = 0.;
pub const AMMUNITION_RESERVE_STATUS_WIDTH: f32 = 12.;
pub const AMMUNITION_RESERVE_STATUS_HEIGHT: f32 = 12.;

pub const AMMUNITION_RESERVE_STATUS_OK_START_REL_X: f32 =
    AMMUNITION_RESERVE_STATUS_OK_START_X / UI_SPRITE_SHEET_WIDTH;
pub const AMMUNITION_RESERVE_STATUS_OK_START_REL_Y: f32 =
    AMMUNITION_RESERVE_STATUS_OK_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const AMMUNITION_RESERVE_STATUS_LOW_START_REL_X: f32 =
    AMMUNITION_RESERVE_STATUS_LOW_START_X / UI_SPRITE_SHEET_WIDTH;
pub const AMMUNITION_RESERVE_STATUS_LOW_START_REL_Y: f32 =
    AMMUNITION_RESERVE_STATUS_LOW_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const AMMUNITION_RESERVE_STATUS_EMPTY_START_REL_X: f32 =
    AMMUNITION_RESERVE_STATUS_EMPTY_START_X / UI_SPRITE_SHEET_WIDTH;
pub const AMMUNITION_RESERVE_STATUS_EMPTY_START_REL_Y: f32 =
    AMMUNITION_RESERVE_STATUS_EMPTY_START_Y / UI_SPRITE_SHEET_HEIGHT;
pub const AMMUNITION_RESERVE_STATUS_REL_WIDTH: f32 =
    AMMUNITION_RESERVE_STATUS_WIDTH / UI_SPRITE_SHEET_WIDTH;
pub const AMMUNITION_RESERVE_STATUS_REL_HEIGHT: f32 =
    AMMUNITION_RESERVE_STATUS_HEIGHT / UI_SPRITE_SHEET_HEIGHT;

#[derive(Debug, Clone)]

pub enum AmmunitionReserveStatus {
    Ok,
    Low,
    Empty,
}

impl AmmunitionReserveStatus {
    pub fn relative_src(&self) -> [f32; 4] {
        match self {
            Self::Ok => [
                AMMUNITION_RESERVE_STATUS_OK_START_REL_X,
                AMMUNITION_RESERVE_STATUS_OK_START_REL_Y,
                AMMUNITION_RESERVE_STATUS_REL_WIDTH,
                AMMUNITION_RESERVE_STATUS_REL_HEIGHT,
            ],
            Self::Low => [
                AMMUNITION_RESERVE_STATUS_LOW_START_REL_X,
                AMMUNITION_RESERVE_STATUS_LOW_START_REL_Y,
                AMMUNITION_RESERVE_STATUS_REL_WIDTH,
                AMMUNITION_RESERVE_STATUS_REL_HEIGHT,
            ],
            Self::Empty => [
                AMMUNITION_RESERVE_STATUS_EMPTY_START_REL_X,
                AMMUNITION_RESERVE_STATUS_EMPTY_START_REL_Y,
                AMMUNITION_RESERVE_STATUS_REL_WIDTH,
                AMMUNITION_RESERVE_STATUS_REL_HEIGHT,
            ],
        }
    }
}
