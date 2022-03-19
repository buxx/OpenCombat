use std::{cmp, collections::HashSet};

use ggez::graphics::Rect;

use crate::{behavior::Behavior, order::PendingOrder, physics::path::find_path, types::*};

use super::Engine;

impl Engine {
    pub fn get_entities_in_area(&self, start: WorldPoint, end: WorldPoint) -> Vec<EntityIndex> {
        let mut entity_indexes = vec![];

        let from = WindowPoint::new(
            cmp::min(start.x as i32, end.x as i32) as f32,
            cmp::min(start.y as i32, end.y as i32) as f32,
        );
        let to = WindowPoint::new(
            cmp::max(start.x as i32, end.x as i32) as f32,
            cmp::max(start.y as i32, end.y as i32) as f32,
        );
        let area = Rect::new(from.x, from.y, to.x - from.x, to.y - from.y);

        for (i, scene_item) in self.shared_state.entities().iter().enumerate() {
            let entity_point = scene_item.get_world_point();
            if area.contains(entity_point.to_vec2()) {
                entity_indexes.push(EntityIndex(i));
            }
        }

        entity_indexes
    }

    pub fn get_entities_at_point(&self, point: WorldPoint) -> Vec<EntityIndex> {
        let mut entity_indexes = vec![];

        for (i, scene_item) in self.shared_state.entities().iter().enumerate() {
            let rect = scene_item.get_selection_rect();
            if rect.contains(point.to_vec2()) {
                entity_indexes.push(EntityIndex(i));
            }
        }

        entity_indexes
    }

    pub fn filter_entities_by_side(&self, entity_indexes: Vec<EntityIndex>) -> Vec<EntityIndex> {
        let mut filtered_entity_indexes = vec![];

        for entity_index in entity_indexes {
            let entity = self.shared_state.entity(entity_index);
            if entity.get_side() == self.local_state.side() {
                filtered_entity_indexes.push(entity_index);
            }
        }

        filtered_entity_indexes
    }

    pub fn _filter_entities_by_visibility(
        &self,
        entity_indexes: Vec<EntityIndex>,
    ) -> Vec<EntityIndex> {
        // TODO
        entity_indexes
    }

    pub fn squad_ids_from_entities(&self, entity_indexes: Vec<EntityIndex>) -> Vec<SquadUuid> {
        let mut all_squad_uuids: Vec<SquadUuid> = entity_indexes
            .iter()
            .map(|i| self.shared_state.entity(*i))
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

    pub fn get_pending_order_params(
        &self,
        pending_order: &PendingOrder,
        squad_id: SquadUuid,
        cached_points: &Vec<WorldPoint>,
    ) -> Vec<(WindowPoint, Angle, Offset)> {
        let squad = self.shared_state.squad(squad_id);
        let squad_leader = self.shared_state.entity(squad.leader());
        let order_marker = pending_order.marker();
        match pending_order {
            PendingOrder::MoveTo | PendingOrder::MoveFastTo | PendingOrder::SneakTo => {
                let mut params = vec![];
                for cached_point in cached_points {
                    params.push((
                        self.local_state
                            .window_point_from_world_point(*cached_point),
                        Angle(0.),
                        order_marker.offset(),
                    ));
                }
                params.push((
                    *self.local_state.get_current_cursor_window_point(),
                    Angle(0.),
                    order_marker.offset(),
                ));
                params
            }
            PendingOrder::Defend | PendingOrder::Hide => {
                let to_point = self.local_state.get_current_cursor_world_point().to_vec2();
                let from_point = squad_leader.get_world_point().to_vec2();
                vec![(
                    self.local_state
                        .window_point_from_world_point(squad_leader.get_world_point()),
                    Angle::from_points(&to_point, &from_point),
                    order_marker.offset(),
                )]
            }
        }
    }

    pub fn create_path_finding(
        &self,
        squad_id: SquadUuid,
        order_marker_index: Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
    ) -> Option<WorldPaths> {
        let squad = self.shared_state.squad(squad_id);
        let entity = self.shared_state.entity(squad.leader());
        let entity_world_point = entity.get_world_point();
        let entity_grid_point = self.grid_point_from_world_point(entity_world_point);
        let cursor_world_point = self.local_state.get_current_cursor_world_point();
        let cursor_grid_point = self.grid_point_from_world_point(cursor_world_point);

        // Determine different path "part" to find:
        // Editing existing case
        let bounds = if let Some(order_marker_index_) = order_marker_index {
            // Create path finding with order_marker_index expect squad currently following world paths. But if not, squad maybe finished its.
            if let Some(current_squad_world_paths) = self.current_squad_world_paths(squad_id) {
                let mut bounds_ = vec![];
                for (squad_order_marker_index, world_path) in
                    current_squad_world_paths.paths.iter().enumerate()
                {
                    let world_start_point = world_path.next_point().expect("Must have points here");
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
                vec![(entity_grid_point, cursor_grid_point)]
            }
        // Some points already cached (append)
        } else if cached_points.len() > 1 {
            let mut last = entity_grid_point;
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
            vec![(entity_grid_point, cursor_grid_point)]
        };

        // Build path finding on each parts
        let mut world_paths = vec![];
        for (bound_start, bound_end) in bounds {
            if let Some(grid_points_path) = find_path(&self.map, &bound_start, &bound_end) {
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
        let squad_leader = self.shared_state.entity(squad.leader());
        match squad_leader.get_behavior() {
            Behavior::MoveTo(world_paths)
            | Behavior::MoveFastTo(world_paths)
            | Behavior::SneakTo(world_paths) => Some(world_paths),
            _ => None,
        }
    }

    pub fn create_world_paths_from_context(
        &self,
        squad_id: SquadUuid,
        order_marker_index: Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
    ) -> Option<WorldPaths> {
        // FIXME BS NOW : order_marker_index : Il faudra remplacer le morceau que l'on deplace
        // FIXME dans get_display_paths ?

        for (display_paths, path_squad_id) in self.local_state.get_display_paths() {
            if *path_squad_id == squad_id {
                return Some(display_paths.clone());
            }
        }

        return self.create_path_finding(squad_id, order_marker_index, cached_points);
    }

    pub fn angle_from_cursor_and_squad(&self, squad_id: SquadUuid) -> Angle {
        let squad = self.shared_state.squad(squad_id);
        let squad_leader = self.shared_state.entity(squad.leader());
        let to_point = self.local_state.get_current_cursor_world_point().to_vec2();
        let from_point = squad_leader.get_world_point().to_vec2();
        Angle::from_points(&to_point, &from_point)
    }
}
