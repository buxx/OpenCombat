use battle_core::{
    behavior::Behavior,
    entity::soldier::Soldier,
    game::cover::CoverFinder,
    order::{marker::OrderMarker, Order, PendingOrder},
    state::battle::message::{BattleStateMessage, SoldierMessage, VehicleMessage},
    types::*,
    utils::DebugPoint,
};

use super::{
    message::{EngineMessage, GuiStateMessage},
    Engine,
};

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
        return Some(Order::Hide(angle));
    }

    pub fn create_engage_order(&self, squad_id: &SquadUuid) -> Option<Order> {
        let world_point = self.gui_state.current_cursor_world_point();
        if let Some(soldier) = self
            .get_opponent_soldiers_at_point(world_point)
            .iter()
            .filter(|s| s.can_be_designed_as_target())
            .filter(|s| {
                self.battle_state
                    .soldier_is_visible_by_side(s, self.gui_state.side())
            })
            .collect::<Vec<&&Soldier>>()
            .first()
        {
            return Some(Order::EngageSquad(soldier.squad_uuid()));
        } else {
            if self.battle_state.point_is_visible_by_squad(
                &self.server_config,
                &world_point,
                squad_id,
            ) {
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
                PendingOrder::MoveFastTo(*squad_index, *order_marker_index, cached_points.clone())
            }
            OrderMarker::SneakTo => {
                PendingOrder::SneakTo(*squad_index, *order_marker_index, cached_points.clone())
            }
            OrderMarker::Defend => PendingOrder::Defend(*squad_index),
            OrderMarker::Hide => PendingOrder::Hide(*squad_index),
            OrderMarker::EngageSquad => PendingOrder::EngageOrFire(*squad_index),
            OrderMarker::SuppressFire => PendingOrder::EngageOrFire(*squad_index),
        }
    }

    pub fn define_order(&self, squad_leader: &SoldierIndex, order: &Order) -> Vec<EngineMessage> {
        let mut messages = vec![EngineMessage::BattleState(BattleStateMessage::Soldier(
            *squad_leader,
            SoldierMessage::SetOrder(order.clone()),
        ))];

        if self.battle_state.phase().placement() {
            // When in placement, solve order immediately
            messages.extend(self.define_placement_order(squad_leader, order))
        }

        messages
    }

    pub fn define_placement_order(
        &self,
        squad_leader: &SoldierIndex,
        order: &Order,
    ) -> Vec<EngineMessage> {
        if let Some(vehicle_index) = self.battle_state.soldier_vehicle(*squad_leader) {
            self.define_vehicle_placement_order(&vehicle_index, squad_leader, order)
        } else {
            self.define_pedestrian_placement_order(squad_leader, order)
        }
    }
    pub fn define_vehicle_placement_order(
        &self,
        vehicle_index: &VehicleIndex,
        _squad_leader: &SoldierIndex,
        order: &Order,
    ) -> Vec<EngineMessage> {
        match order {
            Order::Defend(angle) | Order::Hide(angle) => {
                vec![EngineMessage::BattleState(BattleStateMessage::Vehicle(
                    *vehicle_index,
                    VehicleMessage::SetChassisOrientation(*angle),
                ))]
            }
            Order::EngageSquad(_)
            | Order::SuppressFire(_)
            | Order::Idle
            | Order::MoveTo(_)
            | Order::MoveFastTo(_)
            | Order::SneakTo(_) => {
                // No direct solving in placement for these orders
                vec![]
            }
        }
    }

    pub fn define_pedestrian_placement_order(
        &self,
        squad_leader: &SoldierIndex,
        order: &Order,
    ) -> Vec<EngineMessage> {
        match order {
            Order::Defend(angle) | Order::Hide(angle) => {
                let mut messages = vec![];
                let leader = self.battle_state.soldier(*squad_leader);
                let squad = self.battle_state.squad(leader.squad_uuid());
                let (moves, debug_points) =
                    CoverFinder::new(&self.battle_state, &self.server_config)
                        .find_arbitrary_cover_points(squad, leader);

                // Debug points
                messages.extend(debug_points.iter().map(|d| {
                    EngineMessage::GuiState(GuiStateMessage::PushDebugPoint(DebugPoint {
                        frame_i: self.gui_state.frame_i() + 60,
                        point: d.point,
                    }))
                }));

                // Place soldiers to expected position
                messages.extend(moves.iter().map(|(soldier_index, _, move_to)| {
                    EngineMessage::BattleState(BattleStateMessage::Soldier(
                        *soldier_index,
                        SoldierMessage::SetWorldPosition(*move_to),
                    ))
                }));

                // Set soldiers orientation
                messages.extend(
                    [
                        vec![(*squad_leader, *angle)],
                        moves
                            .iter()
                            .map(|(soldier_index, _, _)| (*soldier_index, *angle))
                            .collect(),
                    ]
                    .concat()
                    .iter()
                    .map(|(soldier_index, angle)| {
                        EngineMessage::BattleState(BattleStateMessage::Soldier(
                            *soldier_index,
                            SoldierMessage::SetOrientation(*angle),
                        ))
                    }),
                );

                // Set soldiers behavior
                let behavior = match order {
                    Order::Defend(angle) => Behavior::Defend(*angle),
                    Order::Hide(angle) => Behavior::Hide(*angle),
                    _ => unreachable!(),
                };
                messages.extend(
                    [
                        vec![(*squad_leader, behavior.clone())],
                        moves
                            .iter()
                            .map(|(soldier_index, _, _)| (*soldier_index, behavior.clone()))
                            .collect(),
                    ]
                    .concat()
                    .iter()
                    .map(|(soldier_index, behavior)| {
                        EngineMessage::BattleState(BattleStateMessage::Soldier(
                            *soldier_index,
                            SoldierMessage::SetBehavior(behavior.clone()),
                        ))
                    }),
                );

                messages
            }
            Order::EngageSquad(_)
            | Order::SuppressFire(_)
            | Order::Idle
            | Order::MoveTo(_)
            | Order::MoveFastTo(_)
            | Order::SneakTo(_) => {
                // No direct solving in placement for these orders
                vec![]
            }
        }
    }
}
