use std::collections::HashMap;

use bresenham::Bresenham;
use serde::{Deserialize, Serialize};

use crate::{
    config::{ServerConfig, VISIBILITY_FIRSTS, VISIBILITY_PIXEL_STEPS},
    entity::soldier::Soldier,
    map::Map,
    types::{Distance, GridPath, SoldierIndex, WorldPoint},
};

use super::utils::meters_between_world_points;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Visibilities {
    visibilities: HashMap<(SoldierIndex, SoldierIndex), Visibility>,
}

impl Visibilities {
    pub fn new() -> Self {
        Self {
            visibilities: HashMap::new(),
        }
    }

    pub fn set(&mut self, value: HashMap<(SoldierIndex, SoldierIndex), Visibility>) {
        self.visibilities = value;
    }

    pub fn get(&self, soldiers: &(SoldierIndex, SoldierIndex)) -> Option<&Visibility> {
        self.visibilities.get(soldiers)
    }

    pub fn visibles_soldiers_by_soldier(&self, soldier: &Soldier) -> Vec<&Visibility> {
        self.visibilities
            .values()
            .filter(|v| v.from_soldier == soldier.uuid() && v.to_soldier.is_some() && v.visible)
            .collect()
    }

    pub fn visibles_soldiers(&self) -> Vec<&Visibility> {
        self.visibilities
            .values()
            .filter(|v| v.to_soldier.is_some() && v.visible)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.visibilities.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visibility {
    pub from: WorldPoint,
    pub from_soldier: SoldierIndex,
    pub to: WorldPoint,
    pub to_soldier: Option<SoldierIndex>,
    pub path_final_opacity: f32,
    pub to_scene_item_opacity: f32,
    pub opacity_segments: Vec<(WorldPoint, f32)>,
    pub visible: bool,
    pub distance: Distance,
}

impl Visibility {
    pub fn between_soldiers(
        frame_i: u64,
        config: &ServerConfig,
        from_soldier: &Soldier,
        to_soldier: &Soldier,
        map: &Map,
    ) -> Self {
        let from_point = from_soldier.get_world_point();
        let to_point = to_soldier.get_world_point();
        let last_shoot_frame_i = to_soldier.last_shoot_frame_i();

        let by_behavior_modifier: f32 = config.visibility_behavior_modifier(to_soldier.behavior());

        let exclude_lasts = if last_shoot_frame_i + config.visibility_by_last_frame_shoot >= frame_i
        {
            config.visibility_by_last_frame_shoot_distance
        } else {
            0
        };

        let (mut to_soldier_item_opacity, opacity_segments, path_final_opacity) =
            Visibility::between_points(
                config,
                &from_point,
                &to_point,
                map,
                config.visibility_firsts,
                exclude_lasts,
            );

        to_soldier_item_opacity = to_soldier_item_opacity - by_behavior_modifier;
        let visible = to_soldier_item_opacity < config.visible_starts_at;

        let distance = meters_between_world_points(
            &from_soldier.get_world_point(),
            &to_soldier.get_world_point(),
        );
        Self {
            from: from_point,
            from_soldier: from_soldier.uuid(),
            to: to_point,
            to_soldier: Some(to_soldier.uuid()),
            opacity_segments,
            path_final_opacity,
            to_scene_item_opacity: to_soldier_item_opacity,
            visible,
            distance,
        }
    }

    pub fn between_soldier_and_point(
        config: &ServerConfig,
        from_soldier: &Soldier,
        to_point: &WorldPoint,
        map: &Map,
    ) -> Self {
        let from_point = from_soldier.get_world_point();

        let (to_soldier_item_opacity, opacity_segments, path_final_opacity) =
            Visibility::between_points(config, &from_point, &to_point, map, VISIBILITY_FIRSTS, 0);

        let visible = to_soldier_item_opacity < 0.5;
        let distance = meters_between_world_points(&from_point, &to_point);
        Self {
            from: from_point,
            from_soldier: from_soldier.uuid(),
            to: *to_point,
            to_soldier: None,
            opacity_segments,
            path_final_opacity,
            to_scene_item_opacity: to_soldier_item_opacity,
            visible,
            distance,
        }
    }

    // TODO : Optimize performances here
    fn between_points(
        config: &ServerConfig,
        from_point: &WorldPoint,
        to_point: &WorldPoint,
        map: &Map,
        exclude_firsts: usize,
        exclude_lasts: usize,
    ) -> (f32, Vec<(WorldPoint, f32)>, f32) {
        let mut opacity_segments: Vec<(WorldPoint, f32)> = vec![];
        let mut path_final_opacity: f32 = 0.0;
        let mut to_opacity: f32 = 0.0;
        let _visible_by_bullet_fire = false;

        // Compute line pixels
        let pixels = Bresenham::new(
            (from_point.x as isize, from_point.y as isize),
            (to_point.x as isize, to_point.y as isize),
        );

        let mut grid_path: GridPath = GridPath::new();
        let mut other: Vec<(WorldPoint, f32)> = vec![];
        for (pixel_x, pixel_y) in pixels.step_by(VISIBILITY_PIXEL_STEPS) {
            let grid_point =
                map.grid_point_from_world_point(&WorldPoint::new(pixel_x as f32, pixel_y as f32));
            if !grid_path.contains(&grid_point) {
                let terrain_tile = match map
                    .terrain_tiles()
                    .get((grid_point.y * map.width() as i32 + grid_point.x) as usize)
                {
                    Some(tile) => tile,
                    None => {
                        continue;
                    }
                };
                let grid_point_opacity = if grid_path.len() <= exclude_firsts {
                    0.0
                } else {
                    config.terrain_tile_opacity(&terrain_tile.type_)
                };
                grid_path.push(grid_point);
                other.push((
                    WorldPoint::new(pixel_x as f32, pixel_y as f32),
                    grid_point_opacity,
                ));
            }
        }

        let exclude_opacity_starts_at = grid_path.len() - exclude_lasts;
        for (i, (_, (world_point, opacity))) in grid_path.points.iter().zip(other).enumerate() {
            // Disable to_scene_item firsts if seen because firing
            let opacity = if i < exclude_opacity_starts_at {
                opacity
            } else {
                0.
            };
            path_final_opacity += opacity;
            to_opacity += opacity;
            opacity_segments.push((world_point, path_final_opacity));
        }

        (to_opacity, opacity_segments, path_final_opacity)
    }
}
