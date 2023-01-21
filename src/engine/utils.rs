use std::{cmp, collections::HashSet};

use ggez::graphics::Rect;

use crate::{
    behavior::{Behavior, BehaviorMode},
    entity::vehicle::OnBoardPlace,
    game::Side,
    order::{marker::OrderMarker, Order},
    physics::path::{find_path, Direction, PathMode},
    types::*,
    utils::{angle, WorldShape},
};

use super::Engine;

impl Engine {
    pub fn get_entities_in_area(&self, start: WorldPoint, end: WorldPoint) -> Vec<SoldierIndex> {
        let mut soldier_indexes = vec![];

        let from = WindowPoint::new(
            cmp::min(start.x as i32, end.x as i32) as f32,
            cmp::min(start.y as i32, end.y as i32) as f32,
        );
        let to = WindowPoint::new(
            cmp::max(start.x as i32, end.x as i32) as f32,
            cmp::max(start.y as i32, end.y as i32) as f32,
        );
        let area = Rect::new(from.x, from.y, to.x - from.x, to.y - from.y);

        for (i, scene_item) in self.shared_state.soldiers().iter().enumerate() {
            let soldier_point = scene_item.get_world_point();
            if area.contains(soldier_point.to_vec2()) {
                soldier_indexes.push(SoldierIndex(i));
            }
        }

        soldier_indexes
    }

    pub fn get_soldiers_at_point(&self, point: WorldPoint) -> Vec<SoldierIndex> {
        let mut soldier_indexes = vec![];

        for (i, soldier) in self.shared_state.soldiers().iter().enumerate() {
            let rect = soldier.get_selection_rect();
            if rect.contains(point.to_vec2()) {
                soldier_indexes.push(SoldierIndex(i));
            }
        }

        soldier_indexes
    }

    pub fn filter_entities_by_side(
        &self,
        soldier_indexes: Vec<SoldierIndex>,
        side: &Side,
    ) -> Vec<SoldierIndex> {
        let mut filtered_soldier_indexes = vec![];

        for soldier_index in soldier_indexes {
            let soldier = self.shared_state.soldier(soldier_index);
            if soldier.get_side() == side {
                filtered_soldier_indexes.push(soldier_index);
            }
        }

        filtered_soldier_indexes
    }

    pub fn _filter_entities_by_visibility(
        &self,
        soldier_indexes: Vec<SoldierIndex>,
    ) -> Vec<SoldierIndex> {
        // TODO
        soldier_indexes
    }

    pub fn squad_ids_from_entities(&self, soldier_indexes: Vec<SoldierIndex>) -> Vec<SquadUuid> {
        let mut all_squad_uuids: Vec<SquadUuid> = soldier_indexes
            .iter()
            .map(|i| self.shared_state.soldier(*i))
            .map(|e| e.squad_uuid())
            .collect();
        let unique_squad_uuids: HashSet<SquadUuid> = all_squad_uuids.drain(..).collect();
        unique_squad_uuids.into_iter().collect()
    }

    pub fn grid_point_from_world_point(&self, world_point: WorldPoint) -> GridPoint {
        let x = world_point.x as u32 / self.map.terrain.tileset.tile_width;
        let y = world_point.y as u32 / self.map.terrain.tileset.tile_height;
        GridPoint::new(x as i32, y as i32)
    }

    pub fn world_point_from_grid_point(&self, grid_point: GridPoint) -> WorldPoint {
        let x = (grid_point.x * self.map.terrain.tileset.tile_width as i32)
            + (self.map.terrain.tileset.tile_width as i32 / 2);
        let y = (grid_point.y * self.map.terrain.tileset.tile_height as i32)
            + (self.map.terrain.tileset.tile_height as i32 / 2);
        WorldPoint::new(x as f32, y as f32)
    }

    pub fn create_path_finding(
        &self,
        squad_id: SquadUuid,
        order_marker_index: &Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
        path_mode: &PathMode,
        start_direction: &Option<Direction>,
    ) -> Option<WorldPaths> {
        let squad = self.shared_state.squad(squad_id);
        let soldier = self.shared_state.soldier(squad.leader());
        let soldier_world_point = soldier.get_world_point();
        let soldier_grid_point = self.grid_point_from_world_point(soldier_world_point);
        let cursor_world_point = self.local_state.get_current_cursor_world_point();
        let cursor_grid_point = self.grid_point_from_world_point(cursor_world_point);

        // Prevent compute same thing each frames when path not found
        if &Some(cursor_world_point) == self.local_state.last_computed_path_point() {
            return None;
        }

        // Determine different path "part" to find:
        // Editing existing case
        let bounds = if let Some(order_marker_index_) = order_marker_index {
            // Create path finding with order_marker_index expect squad currently following world paths. But if not, squad maybe finished its.
            if let Some(current_squad_world_paths) = self.current_squad_world_paths(squad_id) {
                let mut bounds_ = vec![];
                for (squad_order_marker_index, world_path) in
                    current_squad_world_paths.paths.iter().enumerate()
                {
                    // This part first point is the current cursor if this part is following edited part
                    let world_start_point = if squad_order_marker_index > 0
                        && order_marker_index_.0 == squad_order_marker_index - 1
                    {
                        cursor_world_point
                    } else {
                        world_path.next_point().expect("Must have points here")
                    };
                    // If we are editing this order marker index, cursor is the end point
                    let world_end_point = if order_marker_index_.0 == squad_order_marker_index {
                        cursor_world_point
                    } else {
                        world_path.last_point().expect("Must have points here")
                    };
                    let start_grid_point = self.grid_point_from_world_point(world_start_point);
                    let end_grid_point = self.grid_point_from_world_point(world_end_point);

                    bounds_.push((start_grid_point, end_grid_point));
                }
                bounds_
            } else {
                vec![(soldier_grid_point, cursor_grid_point)]
            }
        // Some points already cached (append)
        } else if cached_points.len() > 1 {
            let mut last = soldier_grid_point;
            let mut bounds_ = vec![];
            for cached_point in cached_points {
                let grid_cached_point = self.grid_point_from_world_point(*cached_point);
                bounds_.push((last, grid_cached_point));
                last = grid_cached_point;
            }
            bounds_.push((last, cursor_grid_point));
            bounds_
        // First point
        } else {
            vec![(soldier_grid_point, cursor_grid_point)]
        };

        // Build path finding on each parts
        let mut world_paths = vec![];
        for (bound_start, bound_end) in bounds {
            if let Some(grid_points_path) = find_path(
                &self.map,
                &bound_start,
                &bound_end,
                true,
                path_mode,
                start_direction,
            ) {
                if grid_points_path.len() > 0 {
                    let world_point_path = grid_points_path
                        .iter()
                        .map(|p| self.world_point_from_grid_point(GridPoint::from(*p)))
                        .collect();
                    let world_path = WorldPath::new(world_point_path);
                    world_paths.push(world_path);
                }
            }
        }

        if world_paths.len() > 0 {
            return Some(WorldPaths::new(world_paths));
        }

        None
    }

    pub fn current_squad_world_paths(&self, squad_id: SquadUuid) -> Option<&WorldPaths> {
        let squad = self.shared_state.squad(squad_id);
        match self.squad_behavior_mode(&squad_id) {
            BehaviorMode::Ground => {
                return self
                    .shared_state
                    .soldier(squad.leader())
                    .behavior()
                    .world_paths()
            }
            BehaviorMode::Vehicle => {
                // TODO : refact
                if let Some(vehicle_index) = self.shared_state.soldier_vehicle(squad.leader()) {
                    if let Some(board) = self.shared_state.vehicle_board().get(&vehicle_index) {
                        for (place, soldier_index) in board {
                            if place == &OnBoardPlace::Driver {
                                return self
                                    .shared_state
                                    .soldier(*soldier_index)
                                    .behavior()
                                    .world_paths();
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn create_world_paths_from_context(
        &self,
        squad_id: &SquadUuid,
        order_marker_index: &Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
    ) -> Option<WorldPaths> {
        // Take path from displayed path if exist
        for (display_paths, path_squad_id) in self.local_state.get_display_paths() {
            if *path_squad_id == *squad_id {
                return Some(display_paths.clone());
            }
        }

        // Else, create a path
        let (path_mode, start_direction) =
            self.shared_state.squad_path_mode_and_direction(*squad_id);
        return self.create_path_finding(
            *squad_id,
            order_marker_index,
            cached_points,
            &path_mode,
            &start_direction,
        );
    }

    pub fn angle_from_cursor_and_squad(&self, squad_id: SquadUuid) -> Angle {
        let squad = self.shared_state.squad(squad_id);
        let squad_leader = self.shared_state.soldier(squad.leader());
        let to_point = self.local_state.get_current_cursor_world_point().to_vec2();
        let from_point = squad_leader.get_world_point().to_vec2();
        Angle::from_points(&to_point, &from_point)
    }

    pub fn order_marker_selection_shape(
        &self,
        order: &Order,
        marker: &OrderMarker,
        from_point: &WorldPoint,
    ) -> WorldShape {
        WorldShape::from_rect(&marker.sprite_info().selection_rect(*from_point))
            .rotate(&order.angle().unwrap_or(Angle::zero()))
            .cut(marker.selectable())
    }

    pub fn behavior_angle(
        &self,
        behavior: &Behavior,
        reference_point: &WorldPoint,
    ) -> Option<Angle> {
        match behavior {
            Behavior::Idle => None,
            Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => {
                if let Some(next_point) = paths.next_point() {
                    Some(angle(&next_point, &reference_point))
                } else {
                    None
                }
            }
            Behavior::Defend(angle) => Some(*angle),
            Behavior::Hide(angle) => Some(*angle),
            Behavior::DriveTo(_) => None,
            Behavior::RotateTo(_) => None,
            Behavior::SuppressFire(point) => Some(angle(&point, &reference_point)),
            // FIXME BS NOW : don't know yet which soldier ?! :(
            Behavior::EngageSoldier(_) => None,
            // TODO: keep angle for dead/unconscious soldiers
            Behavior::Dead | Behavior::Unconscious => None,
        }
    }
}
