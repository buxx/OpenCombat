use crate::config::{GRID_TILE_HEIGHT, GRID_TILE_WIDTH};
use crate::physics::GridPosition;
use crate::{ScenePoint, WindowPoint};

pub fn grid_position_from_scene_point(position: &ScenePoint) -> GridPosition {
    GridPosition::new(
        (position.x / GRID_TILE_WIDTH) as i32,
        (position.y / GRID_TILE_HEIGHT) as i32,
    )
}

pub fn scene_point_from_window_point(
    window_point: &WindowPoint,
    display_offset: &WindowPoint,
) -> ScenePoint {
    ScenePoint::new(
        window_point.x - display_offset.x,
        window_point.y - display_offset.y,
    )
}

pub fn window_point_from_scene_point(
    scene_point: &ScenePoint,
    display_offset: &WindowPoint,
) -> WindowPoint {
    WindowPoint::new(
        scene_point.x + display_offset.x,
        scene_point.y + display_offset.y,
    )
}
