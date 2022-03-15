use std::collections::HashSet;

use ggez::graphics::Rect;

use crate::types::*;

use super::Engine;

impl Engine {
    pub fn get_entities_in_area(&self, start: WorldPoint, end: WorldPoint) -> Vec<EntityIndex> {
        let mut entity_indexes = vec![];

        let (start, end) = if start.x > end.x {
            (end, start)
        } else {
            (start, end)
        };
        let area = Rect::new(start.x, start.y, end.x - start.x, end.y - start.y);

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

    pub fn filter_entities_by_visibility(
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
}
