use std::{cmp, collections::HashSet};

use ggez::graphics::Rect;

use crate::{order::PendingOrder, physics::path::find_path, types::*};

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
    ) -> (WindowPoint, Angle, Offset) {
        let squad = self.shared_state.squad(squad_id);
        let squad_leader = self.shared_state.entity(squad.leader());
        let order_marker = pending_order.marker();
        let (draw_to, angle) = match pending_order {
            PendingOrder::MoveTo | PendingOrder::MoveFastTo | PendingOrder::SneakTo => (
                *self.local_state.get_current_cursor_window_point(),
                Angle(0.),
            ),
            PendingOrder::Defend | PendingOrder::Hide => {
                let to_point = self.local_state.get_current_cursor_world_point().to_vec2();
                let from_point = squad_leader.get_world_point().to_vec2();
                (
                    self.local_state
                        .window_point_from_world_point(squad_leader.get_world_point()),
                    Angle::from_points(&to_point, &from_point),
                )
            }
        };
        let offset = order_marker.offset();

        (draw_to, angle, offset)
    }

    pub fn create_path_finding(&self, squad_id: SquadUuid) -> Option<WorldPaths> {
        let squad = self.shared_state.squad(squad_id);
        let entity = self.shared_state.entity(squad.leader());
        let entity_world_point = entity.get_world_point();
        let entity_grid_point = self.grid_point_from_world_point(entity_world_point);
        let cursor_world_point = self.local_state.get_current_cursor_world_point();
        let cursor_grid_point = self.grid_point_from_world_point(cursor_world_point);

        let grid_point_path =
            find_path(&self.map, &entity_grid_point, &cursor_grid_point).unwrap_or(vec![]);
        if grid_point_path.len() > 0 {
            let world_point_path = grid_point_path
                .iter()
                .map(|p| self.world_point_from_grid_point(GridPoint::from(*p)))
                .collect();
            let world_path = WorldPath::new(world_point_path);
            return Some(WorldPaths::new(vec![world_path]));
        }

        None
    }

    pub fn create_world_paths_from_context(&self, squad_id: SquadUuid) -> Option<WorldPaths> {
        for (display_paths, path_squad_id) in self.local_state.get_display_paths() {
            if *path_squad_id == squad_id {
                return Some(display_paths.clone());
            }
        }

        return self.create_path_finding(squad_id);
    }

    pub fn angle_from_cursor_and_squad(&self, squad_id: SquadUuid) -> Angle {
        let squad = self.shared_state.squad(squad_id);
        let squad_leader = self.shared_state.entity(squad.leader());
        let to_point = self.local_state.get_current_cursor_world_point().to_vec2();
        let from_point = squad_leader.get_world_point().to_vec2();
        Angle::from_points(&to_point, &from_point)
    }
}
