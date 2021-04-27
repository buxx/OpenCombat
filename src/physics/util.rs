use crate::physics::position::GridPosition;
use crate::{Point2, Vector2, GRID_TILE_HEIGHT, GRID_TILE_WIDTH};

pub fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

pub fn grid_position_from_position(position: &Point2) -> GridPosition {
    GridPosition::new(
        (position.x / GRID_TILE_WIDTH) as i32,
        (position.y / GRID_TILE_HEIGHT) as i32,
    )
}
