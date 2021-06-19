use crate::config::DISTANCE_TO_METERS_COEFFICIENT;
use crate::map::Map;
use crate::physics::GridPoint;
use crate::{Angle, ScenePoint, WindowPoint};

pub fn grid_point_from_scene_point(position: &ScenePoint, map: &Map) -> GridPoint {
    GridPoint::new(
        (position.x / map.terrain.tileset.tile_width as f32) as i32,
        (position.y / map.terrain.tileset.tile_height as f32) as i32,
    )
}

pub fn scene_point_from_grid_point(grid_point: &GridPoint, map: &Map) -> ScenePoint {
    ScenePoint::new(
        grid_point.x as f32 * map.terrain.tileset.tile_width as f32,
        grid_point.y as f32 * map.terrain.tileset.tile_height as f32,
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

pub fn meters_between_scene_points(
    from_scene_point: &ScenePoint,
    to_scene_point: &ScenePoint,
) -> f32 {
    from_scene_point.distance(*to_scene_point) * DISTANCE_TO_METERS_COEFFICIENT
}

pub fn apply_angle_on_point(
    point_to_rotate: &ScenePoint,
    reference_point: &ScenePoint,
    angle: &Angle,
) -> ScenePoint {
    let sin = f32::sin(*angle);
    let cos = f32::cos(*angle);
    let pt = (
        point_to_rotate.x - reference_point.x,
        point_to_rotate.y - reference_point.y,
    );
    let rotated = (
        reference_point.x + pt.0 * cos - pt.1 * sin,
        reference_point.y + pt.0 * sin + pt.1 * cos,
    );
    ScenePoint::new(rotated.0, rotated.1)
}
