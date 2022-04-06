use crate::{
    game::squad::{squad_positions, Formation},
    map::util::find_cover_grid_point,
    message::*,
    order::Order,
    physics::path::find_path,
    types::*,
};
use rayon::prelude::*;

use super::Engine;

impl Engine {
    // FIXME BS NOW : Soldiers in vehicles must be managed differently than ground soldiers
    pub fn tick_entities(&self) -> Vec<Message> {
        let mut messages = vec![];

        // Entities animation
        if self.local_state.get_frame_i() % self.config.soldier_animate_freq() == 0 {
            let soldier_messages: Vec<Message> = (0..self.shared_state.soldiers().len())
                .into_par_iter()
                .flat_map(|i| self.animate_soldier(SoldierIndex(i)))
                .collect();
            messages.extend(soldier_messages);
        }

        // Entities updates
        if self.local_state.get_frame_i() % self.config.soldier_update_freq() == 0 {
            let soldier_messages: Vec<Message> = (0..self.shared_state.soldiers().len())
                .into_par_iter()
                .flat_map(|i| self.update_soldier(SoldierIndex(i)))
                .collect();
            messages.extend(soldier_messages);
        }

        messages
    }

    pub fn soldier_is_squad_leader(&self, soldier_index: SoldierIndex) -> bool {
        let soldier = self.shared_state.soldier(soldier_index);
        let squad_uuid = soldier.squad_uuid();
        let squad_composition = self.shared_state.squad(squad_uuid);
        let squad_leader = squad_composition.leader();
        squad_leader == soldier_index
    }

    pub fn soldier_can_take_order(&self, _soldier_i: SoldierIndex, _order: &Order) -> bool {
        // TODO : check if order can be applied (e.g. if soldier is not panicking, under fire, etc.)
        true
    }

    pub fn take_order(&self, soldier_index: SoldierIndex, order: &Order) -> Vec<Message> {
        // TODO : behavior must be given to other squad soldiers !!!! other soldiers must can accept it too (under fire etc)
        let mut messages = vec![];
        let soldier = self.shared_state.soldier(soldier_index);
        let behavior_mode = self.soldier_behavior_mode(soldier_index);
        let vehicle_place = self.soldier_vehicle_place(soldier_index);
        let new_behavior = match behavior_mode {
            crate::behavior::BehaviorMode::Ground => order.to_ground_behavior(),
            crate::behavior::BehaviorMode::Vehicle => order.to_vehicle_behavior(
                vehicle_place.expect("must have vehicle place if vehicle behavior mode"),
            ),
        };

        if self.soldier_is_squad_leader(soldier_index) {
            match order {
                Order::MoveTo(_)
                | Order::MoveFastTo(_)
                | Order::SneakTo(_)
                | Order::Defend(_)
                | Order::Hide(_) => {
                    messages.extend(self.squad_leader_propagate_order(soldier.squad_uuid(), &order))
                }
            }
        }

        messages.push(Message::SharedState(SharedStateMessage::Soldier(
            soldier_index,
            SoldierMessage::SetBehavior(new_behavior),
        )));

        messages
    }

    pub fn squad_leader_propagate_order(
        &self,
        squad_uuid: SquadUuid,
        order: &Order,
    ) -> Vec<Message> {
        let mut messages = vec![];

        messages.extend(match order {
            Order::MoveTo(_) | Order::MoveFastTo(_) | Order::SneakTo(_) => {
                self.squad_leader_propagate_move_order(squad_uuid, order)
            }
            Order::Defend(_) => self.squad_leader_propagate_defend_order(squad_uuid, order),
            Order::Hide(_) => self.squad_leader_propagate_defend_order(squad_uuid, order),
        });

        messages
    }

    fn squad_leader_propagate_move_order(
        &self,
        squad_uuid: SquadUuid,
        order: &Order,
    ) -> Vec<Message> {
        let mut messages = vec![];
        let squad = self.shared_state.squad(squad_uuid);

        for (soldier_index, point) in squad_positions(squad, Formation::Line, &self.shared_state) {
            let soldier = self.shared_state.soldier(soldier_index);
            if let Some(grid_path) = find_path(
                &self.map,
                &self.grid_point_from_world_point(soldier.get_world_point()),
                &self.grid_point_from_world_point(point),
                true,
            ) {
                let world_path = grid_path
                    .iter()
                    .map(|p| self.world_point_from_grid_point(*p))
                    .collect();
                let world_paths = WorldPaths::new(vec![WorldPath::new(world_path)]);
                let member_order = match order {
                    Order::MoveTo(_) => Order::MoveTo(world_paths),
                    Order::MoveFastTo(_) => Order::MoveFastTo(world_paths),
                    Order::SneakTo(_) => Order::SneakTo(world_paths),
                    _ => unreachable!(),
                };
                messages.push(Message::SharedState(SharedStateMessage::PushSquadOrder(
                    soldier_index,
                    member_order,
                )))
            }
        }

        messages
    }

    fn squad_leader_propagate_defend_order(
        &self,
        squad_uuid: SquadUuid,
        order: &Order,
    ) -> Vec<Message> {
        let squad = self.shared_state.squad(squad_uuid);
        let mut messages = vec![];
        let mut already_used_cover_grid_points: Vec<GridPoint> = vec![];

        for (member_id, formation_position) in
            squad_positions(squad, Formation::Line, &self.shared_state)
        {
            let soldier = self.shared_state.soldier(member_id);
            let grid_point = self.grid_point_from_world_point(formation_position);
            if let Some((cover_grid_point, debug_grid_points)) =
                find_cover_grid_point(&grid_point, &self.map, &already_used_cover_grid_points)
            {
                for debug_grid_point in debug_grid_points.iter() {
                    // FIXME BS NOW : re implement debug point
                    // messages.push(Message::MainStateMessage(MainStateModifier::NewDebugPoint(
                    //     DebugPoint {
                    //         frame_i: frame_i + 120,
                    //         scene_point: scene_point_from_grid_point(debug_grid_point, map),
                    //     },
                    // )))
                }

                let from_world_point = soldier.get_world_point();
                let cover_world_point = self.world_point_from_grid_point(cover_grid_point);
                already_used_cover_grid_points.push(cover_grid_point);

                messages.push(Message::SharedState(SharedStateMessage::PushSquadOrder(
                    member_id,
                    Order::MoveFastTo(WorldPaths::new(vec![WorldPath::new(vec![
                        from_world_point,
                        cover_world_point,
                    ])])),
                )));

                // if let Some(new_order) = match behavior {
                //     ItemBehavior::Dead | ItemBehavior::Unconscious => None,
                //     ItemBehavior::Standing | ItemBehavior::MoveTo(_, _) => {
                //         Some(Order::MoveTo(cover_scene_point))
                //     }
                //     ItemBehavior::MoveFastTo(_, _) => Some(Order::MoveFastTo(cover_scene_point)),
                //     ItemBehavior::EngageSceneItem(_, _)
                //     | ItemBehavior::EngageGridPoint(_)
                //     | ItemBehavior::HideTo(_, _)
                //     | ItemBehavior::Hide => Some(Order::HideTo(cover_scene_point)),
                // } {
                //     already_used_cover_grid_points.push(cover_grid_point);
                //     messages.push(Message::SceneItemMessage(
                //         member_id,
                //         SceneItemModifier::SetNextOrder(new_order),
                //     ));
                // }
            }
        }

        messages
    }

    fn squad_leader_propagate_hide_order(
        &self,
        squad_uuid: SquadUuid,
        order: &Order,
    ) -> Vec<Message> {
        let mut messages = vec![];

        messages
    }
}
