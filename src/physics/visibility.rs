use std::collections::HashMap;

use bresenham::Bresenham;
use serde::{Deserialize, Serialize};

use crate::{
    behavior::Behavior,
    entity::soldier::Soldier,
    map::Map,
    types::{GridPath, Meters, SoldierIndex, WorldPoint},
};

use super::utils::meters_between_scene_points;

// Visibility computing must consider firsts tiles differently
pub const VISIBILITY_FIRSTS: usize = 4;

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
    // TODO : Optimize performances here
    pub fn between_soldiers(
        _frame_i: u64,
        from_soldier: &Soldier,
        to_soldier: &Soldier,
        map: &Map,
    ) -> Self {
        let from_point = from_soldier.get_world_point();
        let to_point = to_soldier.get_world_point();
        let mut opacity_segments: Vec<(WorldPoint, f32)> = vec![];
        let mut grid_path: GridPath = GridPath::new(vec![]);
        let mut path_final_opacity: f32 = 0.0;
        let mut to_soldier_item_opacity: f32 = 0.0;
        let _visible_by_bullet_fire = false;
        // let visible_by_bullet_fire =
        //     if let Some(last_bullet_fire_frame_i) = to_scene_item.last_bullet_fire {
        //         frame_i - last_bullet_fire_frame_i < 240
        //     } else {
        //         false
        //     };

        // Compute line pixels
        let pixels = Bresenham::new(
            (from_point.x as isize, from_point.y as isize),
            (to_point.x as isize, to_point.y as isize),
        );

        // Compute opacity segments
        for (pixel_x, pixel_y) in pixels {
            let grid_point =
                map.grid_point_from_world_point(&WorldPoint::new(pixel_x as f32, pixel_y as f32));
            if !grid_path.contains(&grid_point) {
                let terrain_tile = match map
                    .terrain
                    .tiles
                    .get(&(grid_point.x as u32, grid_point.y as u32))
                {
                    Some(tile) => tile,
                    None => {
                        // println!(
                        //     "Error : Grid point {}.{} is out of map !",
                        //     grid_point.x, grid_point.y
                        // );
                        continue;
                    }
                };
                // Firsts tiles opacity are ignored
                let grid_point_opacity = if grid_path.len() <= VISIBILITY_FIRSTS {
                    0.0
                } else {
                    terrain_tile.opacity
                };
                path_final_opacity += grid_point_opacity;
                to_soldier_item_opacity += grid_point_opacity;
                grid_path.push(grid_point);
                opacity_segments.push((
                    WorldPoint::new(pixel_x as f32, pixel_y as f32),
                    path_final_opacity,
                ));
            }
        }

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

        let by_behavior_modifier: f32 = match to_soldier.get_behavior() {
            Behavior::Idle => 0.5,
            Behavior::Hide(_) => -0.3,
            Behavior::Defend(_) => -0.3,
            Behavior::MoveTo(_) => 1.0,
            Behavior::MoveFastTo(_) => 2.0,
            Behavior::SneakTo(_) => -0.3,
            // TODO : What about vehicle ?
            Behavior::DriveTo(_) => 0.5,
            Behavior::RotateTo(_) => 0.5,
            Behavior::CommandRotateTo(_) => 0.5,
            Behavior::CommandDriveTo(_) => 0.5,
            // ItemBehavior::Dead => 0.0,
            // ItemBehavior::Unconscious => 0.0,
            // ItemBehavior::Standing => 0.5,
            // ItemBehavior::HideTo(_, _) => -0.3,
            // ItemBehavior::Hide => -0.5,
            // ItemBehavior::MoveTo(_, _) => 1.0,
            // ItemBehavior::MoveFastTo(_, _) => 2.0,
            // ItemBehavior::EngageSceneItem(_, _) => 0.0,
            // ItemBehavior::EngageGridPoint(_) => 0.0,
        };

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
}
