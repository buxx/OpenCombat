use crate::{order::Order, types::*};

use super::Engine;

impl Engine {
    pub fn create_move_to_order(&self, squad_id: SquadUuid) -> Option<Order> {
        if let Some(world_paths) = self.create_world_paths_from_context(squad_id) {
            return Some(Order::MoveTo(world_paths));
        }

        None
    }

    pub fn create_move_fast_to_order(&self, squad_id: SquadUuid) -> Option<Order> {
        if let Some(world_paths) = self.create_world_paths_from_context(squad_id) {
            return Some(Order::MoveTo(world_paths));
        }

        None
    }

    pub fn create_sneak_to_order(&self, squad_id: SquadUuid) -> Option<Order> {
        if let Some(world_paths) = self.create_world_paths_from_context(squad_id) {
            return Some(Order::MoveTo(world_paths));
        }

        None
    }

    pub fn create_defend_order(&self, squad_id: SquadUuid) -> Option<Order> {
        let angle = self.angle_from_cursor_and_squad(squad_id);
        return Some(Order::Defend(angle));
    }

    pub fn create_hide_order(&self, squad_id: SquadUuid) -> Option<Order> {
        let angle = self.angle_from_cursor_and_squad(squad_id);
        return Some(Order::Defend(angle));
    }
}
