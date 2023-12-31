use crate::game::Side;

use super::{soldier::SoldierAnimationType, Sprite};

const SPRITE_SHEET_WIDTH: usize = 256;
const SPRITE_SHEET_HEIGHT: usize = 192;
pub const TILE_WIDTH: usize = 32;
pub const TILE_HEIGHT: usize = 32;

#[derive(Debug)]
pub enum WeaponAnimationType {
    Idle,
    Walking,
    Crawling,
    LyingDown,
    Dead,
}

impl Sprite for WeaponAnimationType {
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

    fn src_y(&self, _: &Side) -> f32 {
        let row = match self {
            WeaponAnimationType::Idle => 0,
            WeaponAnimationType::Walking => 1,
            WeaponAnimationType::Crawling => 2,
            WeaponAnimationType::LyingDown => 3,
            WeaponAnimationType::Dead => 4,
        };

        row as f32 / self.sprite_sheet_row_count() as f32
    }

    fn frame_count(&self) -> usize {
        match self {
            WeaponAnimationType::Idle => 2,
            WeaponAnimationType::Walking => 8,
            WeaponAnimationType::Crawling => 8,
            WeaponAnimationType::LyingDown => 2,
            WeaponAnimationType::Dead => 2,
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
            WeaponAnimationType::Idle => 1.,
            WeaponAnimationType::Walking => 4.,
            WeaponAnimationType::Crawling => 4.,
            WeaponAnimationType::LyingDown => 1.,
            WeaponAnimationType::Dead => 1.,
        }
    }
}

impl From<&SoldierAnimationType> for WeaponAnimationType {
    fn from(value: &SoldierAnimationType) -> Self {
        match value {
            SoldierAnimationType::Idle => Self::Idle,
            SoldierAnimationType::Walking => Self::Walking,
            SoldierAnimationType::Crawling => Self::Crawling,
            SoldierAnimationType::LyingDown => Self::LyingDown,
            SoldierAnimationType::DeadWithSideBlood => Self::Dead,
        }
    }
}
