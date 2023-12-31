use crate::{
    game::Side,
    types::{AbsoluteOffset, RelativeOffset},
    utils::Rect,
};

pub mod explosion;
pub mod soldier;
pub mod vehicle;
pub mod weapon;

pub trait Sprite {
    fn sprite_sheet_column_count(&self) -> usize;
    fn sprite_sheet_row_count(&self) -> usize;
    fn src_x_start(&self) -> f32;
    fn src_x_end(&self) -> f32;
    fn src_y(&self, side: &Side) -> f32;
    fn frame_count(&self) -> usize;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn duration(&self) -> f32;
}

pub struct SpriteInfo {
    pub start_x: f32,
    pub start_y: f32,
    pub tile_width: f32,
    pub tile_height: f32,
    pub relative_start_x: f32,
    pub relative_start_y: f32,
    pub relative_tile_width: f32,
    pub relative_tile_height: f32,
}

impl SpriteInfo {
    pub fn new(
        start_x: f32,
        start_y: f32,
        width: f32,
        height: f32,
        sprite_sheet_width: f32,
        sprite_sheet_height: f32,
    ) -> Self {
        Self {
            start_x,
            start_y,
            tile_width: width,
            tile_height: height,
            relative_start_x: start_x / sprite_sheet_width,
            relative_start_y: start_y / sprite_sheet_height,
            relative_tile_width: width / sprite_sheet_width,
            relative_tile_height: height / sprite_sheet_height,
        }
    }

    pub fn relative_rect(&self) -> Rect {
        Rect::new(
            self.relative_start_x,
            self.relative_start_y,
            self.relative_tile_width,
            self.relative_tile_height,
        )
    }

    pub fn shadow_version(&self) -> Self {
        // Convention is shadow sprite is at right of regular sprite
        Self {
            start_x: self.start_x + self.tile_width,
            start_y: self.start_y + self.tile_height,
            tile_width: self.tile_width,
            tile_height: self.tile_height,
            relative_start_x: self.relative_start_x + self.relative_tile_width,
            relative_start_y: self.relative_start_y,
            relative_tile_width: self.relative_tile_width,
            relative_tile_height: self.relative_tile_height,
        }
    }

    pub fn abs_offset(&self, relative_offset: &RelativeOffset) -> AbsoluteOffset {
        AbsoluteOffset {
            x: self.tile_width * relative_offset.x,
            y: self.tile_height * relative_offset.y,
        }
    }
}
