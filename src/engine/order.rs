use crate::{
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
            OrderMarker::FireTo => todo!(),
            OrderMarker::Defend => PendingOrder::Defend(*squad_index, Angle(0.)),
            OrderMarker::Hide => PendingOrder::Hide(*squad_index, Angle(0.)),
        }
    }
}
