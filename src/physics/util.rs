use crate::map::Map;
use crate::physics::GridPoint;
use crate::{ScenePoint, WindowPoint};

pub fn grid_position_from_scene_point(position: &ScenePoint, map: &Map) -> GridPoint {
    GridPoint::new(
        (position.x / map.terrain.tileset.tile_width as f32) as i32,
        (position.y / map.terrain.tileset.tile_height as f32) as i32,
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
