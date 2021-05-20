use crate::config::VISIBILITY_IGNORE_FIRSTS;
use crate::map::Map;
use crate::physics::util::grid_point_from_scene_point;
use crate::scene::item::SceneItem;
use crate::{GridPath, ScenePoint};
use bresenham::Bresenham;

pub struct Visibility {
    pub from_scene_point: ScenePoint,
    pub to_scene_point: ScenePoint,
    pub total_opacity: f32,
    pub opacity_segments: Vec<(ScenePoint, f32)>,
}

impl Visibility {
    pub fn new(scene_item_from: &SceneItem, to_scene_point: &ScenePoint, map: &Map) -> Self {
        let mut opacity_segments: Vec<(ScenePoint, f32)> = vec![];
        let mut grid_path: GridPath = vec![];
        let mut total_opacity: f32 = 0.0;

        // Compute line pixels
        let pixels = Bresenham::new(
            (
                scene_item_from.position.x as isize,
                scene_item_from.position.y as isize,
            ),
            (to_scene_point.x as isize, to_scene_point.y as isize),
        );

        // Compute opacity segments
        for (pixel_x, pixel_y) in pixels {
            let grid_point =
                grid_point_from_scene_point(&ScenePoint::new(pixel_x as f32, pixel_y as f32), map);
            if !grid_path.contains(&grid_point) {
                let terrain_tile = map
                    .terrain
                    .tiles
                    .get(&(grid_point.x as u32, grid_point.y as u32))
                    .expect("Work with path only in map !");
                // Firsts tiles opacity are ignored
                let grid_point_opacity = if grid_path.len() <= VISIBILITY_IGNORE_FIRSTS {
                    0.0
                } else {
                    terrain_tile.opacity
                };
                total_opacity += grid_point_opacity;
                grid_path.push(grid_point);
                opacity_segments.push((
                    ScenePoint::new(pixel_x as f32, pixel_y as f32),
                    total_opacity,
                ));
            }
        }

        Self {
            from_scene_point: scene_item_from.position,
            to_scene_point: to_scene_point.clone(),
            opacity_segments,
            total_opacity,
        }
    }
}
