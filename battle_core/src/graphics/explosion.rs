use super::Sprite;

const SPRITE_SHEET_WIDTH: usize = 2156;
const SPRITE_SHEET_HEIGHT: usize = 190;
pub const TILE_WIDTH: usize = 196;
pub const TILE_HEIGHT: usize = 190;

#[derive(Debug)]
pub enum ExplosionAnimationType {
    Explosion1,
}

impl Sprite for ExplosionAnimationType {
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

    fn src_y(&self) -> f32 {
        let row = match self {
            ExplosionAnimationType::Explosion1 => 0,
        };

        row as f32 / self.sprite_sheet_row_count() as f32
    }

    fn frame_count(&self) -> usize {
        match self {
            ExplosionAnimationType::Explosion1 => 12,
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
            ExplosionAnimationType::Explosion1 => 1.5,
        }
    }
}
