use super::animation::Sprite;

#[derive(Debug)]
pub enum SoldierAnimationType {
    Idle,
    Walking,
    Running,
}

impl Sprite for SoldierAnimationType {
    fn sprite_sheet_column_count(&self) -> usize {
        8
    }
    fn sprite_sheet_row_count(&self) -> usize {
        2
    }

    fn src_x_start(&self) -> f32 {
        0.
    }

    fn src_x_end(&self) -> f32 {
        (self.frame_count() - 1) as f32 / self.sprite_sheet_column_count() as f32
    }

    fn src_y(&self) -> f32 {
        let row = match self {
            SoldierAnimationType::Idle => 0,
            SoldierAnimationType::Walking => 1,
            SoldierAnimationType::Running => 1,
        };

        row as f32 / self.sprite_sheet_row_count() as f32
    }

    fn frame_count(&self) -> usize {
        match self {
            SoldierAnimationType::Idle => 2,
            SoldierAnimationType::Walking => 8,
            SoldierAnimationType::Running => 8,
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
            SoldierAnimationType::Running => 2.,
        }
    }
}
