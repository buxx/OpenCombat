use std::collections::HashMap;

use bresenham::Bresenham;
use serde::{Deserialize, Serialize};

use crate::{
    behavior::Behavior,
    config::{VISIBILITY_FIRSTS, VISIBILITY_PIXEL_STEPS},
    entity::soldier::Soldier,
    map::Map,
    types::{GridPath, Meters, SoldierIndex, WorldPoint},
};

use super::utils::meters_between_scene_points;

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
    pub distance: Meters,
}

impl Visibility {
    pub fn between_soldiers(
        frame_i: u64,
        from_soldier: &Soldier,
        to_soldier: &Soldier,
        map: &Map,
    ) -> Self {
        let from_point = from_soldier.get_world_point();
        let to_point = to_soldier.get_world_point();

        let by_behavior_modifier: f32 = match to_soldier.behavior() {
            Behavior::Idle => 0.5,
            // Behavior::EngageSoldier(_) => 0.5,
            Behavior::Hide(_) => -0.3,
            Behavior::Defend(_) => -0.3,
            Behavior::MoveTo(_) => 1.0,
            Behavior::MoveFastTo(_) => 2.0,
            Behavior::SneakTo(_) => -0.3,
            // TODO : What about vehicle ?
            Behavior::DriveTo(_) => 0.5,
            Behavior::RotateTo(_) => 0.5,
            Behavior::SuppressFire(_) => 0.5,
            Behavior::EngageSoldier(_) => 0.5,
            // ItemBehavior::Dead => 0.0,
            // ItemBehavior::Unconscious => 0.0,
            // ItemBehavior::Standing => 0.5,
            // ItemBehavior::HideTo(_, _) => -0.3,
            // ItemBehavior::Hide => -0.5,
            // ItemBehavior::MoveTo(_, _) => 1.0,
            // ItemBehavior::MoveFastTo(_, _) => 2.0,
            // ItemBehavior::EngageSceneItem(_, _) => 0.0,
            // ItemBehavior::EngageGridPoint(_) => 0.0,
            Behavior::Dead => 1000.0, // Always visible
            Behavior::Unconscious => 1000.0,
        };

        let (mut to_soldier_item_opacity, opacity_segments, path_final_opacity) =
            Visibility::_between_points(frame_i, &from_point, &to_point, map, VISIBILITY_FIRSTS);

        to_soldier_item_opacity = to_soldier_item_opacity - by_behavior_modifier;
        let visible = to_soldier_item_opacity < 0.5;

        let distance = meters_between_scene_points(
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
        frame_i: u64,
        from_soldier: &Soldier,
        to_point: &WorldPoint,
        map: &Map,
    ) -> Self {
        let from_point = from_soldier.get_world_point();

        let (to_soldier_item_opacity, opacity_segments, path_final_opacity) =
            Visibility::_between_points(frame_i, &from_point, &to_point, map, VISIBILITY_FIRSTS);

        let visible = to_soldier_item_opacity < 0.5;
        let distance = meters_between_scene_points(&from_point, &to_point);
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
    fn _between_points(
        _frame_i: u64,
        from_point: &WorldPoint,
        to_point: &WorldPoint,
        map: &Map,
        exclude_firsts: usize,
    ) -> (f32, Vec<(WorldPoint, f32)>, f32) {
        let mut opacity_segments: Vec<(WorldPoint, f32)> = vec![];
        let mut grid_path: GridPath = GridPath::new();
        let mut path_final_opacity: f32 = 0.0;
        let mut to_opacity: f32 = 0.0;
        let _visible_by_bullet_fire = false;
        // let visible_by_bullet_fire =
        //     if let Some(last_bullet_fire_frame_i) = to_scene_item.last_bullet_fire {
        //         frame_i - last_bullet_fire_frame_i < 240
        //     } else {
        //         false
        //     };

        // Disable to_scene_item firsts if seen because firing
        // if visible_by_bullet_fire {
        //     let start_from = grid_path.len() - cmp::min(grid_path.len(), VISIBILITY_FIRSTS);
        //     for grid_point in grid_path[start_from..].iter() {
        //         let terrain_tile = map
        //             .terrain
        //             .tiles
        //             .get(&(grid_point.x as u32, grid_point.y as u32))
        //             .expect("Work with path only in map !");
        //         to_scene_item_opacity -= terrain_tile.opacity;
        //     }
        // }

        // Compute line pixels
        let pixels = Bresenham::new(
            (from_point.x as isize, from_point.y as isize),
            (to_point.x as isize, to_point.y as isize),
        );

        // Compute opacity segments
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
                // Firsts tiles opacity are ignored
                let grid_point_opacity = if grid_path.len() <= exclude_firsts {
                    0.0
                } else {
                    terrain_tile.opacity
                };
                path_final_opacity += grid_point_opacity;
                to_opacity += grid_point_opacity;
                grid_path.push(grid_point);
                opacity_segments.push((
                    WorldPoint::new(pixel_x as f32, pixel_y as f32),
                    path_final_opacity,
                ));
            }
        }

        (to_opacity, opacity_segments, path_final_opacity)
    }
}
