use crate::game::Side;

use super::Sprite;
use serde::{Deserialize, Serialize};

pub const SIDE_B_Y_OFFSET: usize = 6;
const SPRITE_SHEET_WIDTH: usize = 512;
const SPRITE_SHEET_HEIGHT: usize = 768;
pub const TILE_WIDTH: usize = 64;
pub const TILE_HEIGHT: usize = 64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SoldierAnimationType {
    Idle,
    Walking,
    Crawling,
    LyingDown,
    DeadWithSideBlood,
}

impl Sprite for SoldierAnimationType {
    fn sprite_sheet_column_count(&self) -> usize {
        SPRITE_SHEET_WIDTH / TILE_WIDTH
    }
    fn sprite_sheet_row_count(&self) -> usize {
        SPRITE_SHEET_HEIGHT / TILE_HEIGHT
    }

    fn src_x_start(&self) -> f32 {
        0.
    }

    fn src_x_end(&self) -> f32 {
        (self.frame_count() - 1) as f32 / self.sprite_sheet_column_count() as f32
    }

    fn src_y(&self, side: &Side) -> f32 {
        let row = match self {
            SoldierAnimationType::Idle => 0,
            SoldierAnimationType::Walking => 1,
            SoldierAnimationType::Crawling => 2,
            SoldierAnimationType::LyingDown => 3,
            SoldierAnimationType::DeadWithSideBlood => 4,
        };

        let row = if side == &Side::B {
            row + SIDE_B_Y_OFFSET
        } else {
            row
        };

        row as f32 / self.sprite_sheet_row_count() as f32
    }

    fn frame_count(&self) -> usize {
        match self {
            SoldierAnimationType::Idle => 2,
            SoldierAnimationType::Walking => 8,
            SoldierAnimationType::Crawling => 8,
            SoldierAnimationType::LyingDown => 2,
            SoldierAnimationType::DeadWithSideBlood => 2,
        }
    }

    fn width(&self) -> f32 {
        1. / self.sprite_sheet_column_count() as f32
    }

    fn height(&self) -> f32 {
        1. / self.sprite_sheet_row_count() as f32
    }

    fn duration(&self) -> f32 {
        match self {
            SoldierAnimationType::Idle => 1.,
            SoldierAnimationType::Walking => 4.,
            SoldierAnimationType::Crawling => 4.,
            SoldierAnimationType::LyingDown => 1.,
            SoldierAnimationType::DeadWithSideBlood => 1.,
        }
    }
}
