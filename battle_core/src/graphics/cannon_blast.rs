use crate::game::{weapon::WeaponSprite, Side};

use super::{soldier::SoldierAnimationType, Sprite};

const SPRITE_SHEET_WIDTH: usize = 512;
const SPRITE_SHEET_HEIGHT: usize = 384;
pub const TILE_WIDTH: usize = 64;
pub const TILE_HEIGHT: usize = 64;

#[derive(Debug)]
pub enum CannonBlastAnimationType {
    RiffleOneShotOnLying,
}

impl From<(WeaponSprite, SoldierAnimationType)> for CannonBlastAnimationType {
    fn from(value: (WeaponSprite, SoldierAnimationType)) -> Self {
        match value {
            (WeaponSprite::Riffle, SoldierAnimationType::Idle)
            | (WeaponSprite::Riffle, SoldierAnimationType::Walking)
            | (WeaponSprite::Riffle, SoldierAnimationType::Crawling)
            | (WeaponSprite::Riffle, SoldierAnimationType::LyingDown)
            | (WeaponSprite::Riffle, SoldierAnimationType::DeadWithSideBlood) => {
                Self::RiffleOneShotOnLying
            }
        }
    }
}

impl Sprite for CannonBlastAnimationType {
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
            CannonBlastAnimationType::RiffleOneShotOnLying => 0,
        };

        row as f32 / self.sprite_sheet_row_count() as f32
    }

    fn frame_count(&self) -> usize {
        match self {
            CannonBlastAnimationType::RiffleOneShotOnLying => 8,
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
            CannonBlastAnimationType::RiffleOneShotOnLying => 0.18,
        }
    }
}
