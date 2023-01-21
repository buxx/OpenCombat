use crate::{
    entity::soldier::Soldier,
    order::{marker::OrderMarker, Order, PendingOrder},
    types::*,
};

use super::Engine;

impl Engine {
    pub fn create_move_to_order(
        &self,
        squad_id: &SquadUuid,
        order_marker_index: &Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
    ) -> Option<Order> {
        if let Some(world_paths) =
            self.create_world_paths_from_context(squad_id, order_marker_index, cached_points)
        {
            return Some(Order::MoveTo(world_paths));
        }

        None
    }

    pub fn create_move_fast_to_order(
        &self,
        squad_id: &SquadUuid,
        order_marker_index: &Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
    ) -> Option<Order> {
        if let Some(world_paths) =
            self.create_world_paths_from_context(squad_id, order_marker_index, cached_points)
        {
            return Some(Order::MoveFastTo(world_paths));
        }

        None
    }

    pub fn create_sneak_to_order(
        &self,
        squad_id: &SquadUuid,
        order_marker_index: &Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
    ) -> Option<Order> {
        if let Some(world_paths) =
            self.create_world_paths_from_context(squad_id, order_marker_index, cached_points)
        {
            return Some(Order::SneakTo(world_paths));
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

    pub fn create_engage_order(&self, squad_id: &SquadUuid) -> Option<Order> {
        let world_point = self.local_state.get_current_cursor_world_point();
        if let Some(soldier) = self
            .get_opponent_soldiers_at_point(world_point)
            .iter()
            .filter(|s| s.can_be_designed_as_target())
            .filter(|s| self.soldier_is_visible_by_side(s, self.local_state.side()))
            .collect::<Vec<&&Soldier>>()
            .first()
        {
            return Some(Order::EngageSquad(soldier.squad_uuid()));
        } else {
            if self.point_is_visible_by_squad(&world_point, squad_id) {
                return Some(Order::SuppressFire(world_point));
            }
        }

        None
    }

    pub fn create_pending_order_from_order_marker(
        &self,
        order_marker: &OrderMarker,
        squad_index: &SquadUuid,
        order_marker_index: &Option<OrderMarkerIndex>,
        cached_points: &Vec<WorldPoint>,
    ) -> PendingOrder {
        match order_marker {
            OrderMarker::MoveTo => {
                PendingOrder::MoveTo(*squad_index, *order_marker_index, cached_points.clone())
            }
            OrderMarker::MoveFastTo => {
                PendingOrder::MoveTo(*squad_index, *order_marker_index, cached_points.clone())
            }
            OrderMarker::SneakTo => {
                PendingOrder::MoveTo(*squad_index, *order_marker_index, cached_points.clone())
            }
            OrderMarker::Defend => PendingOrder::Defend(*squad_index),
            OrderMarker::Hide => PendingOrder::Hide(*squad_index),
            OrderMarker::EngageSquad => PendingOrder::EngageOrFire(*squad_index),
            OrderMarker::SuppressFire => PendingOrder::EngageOrFire(*squad_index),
        }
    }
}
