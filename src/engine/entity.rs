use crate::{
    game::squad::{squad_positions, Formation},
    message::*,
    order::Order,
    physics::path::find_path,
    types::*,
};
use rayon::prelude::*;

use super::Engine;

impl Engine {
    pub fn tick_entities(&self) -> Vec<Message> {
        let mut messages = vec![];

        // Entities animation
        if self.local_state.get_frame_i() % self.config.entity_animate_freq() == 0 {
            let entity_messages: Vec<Message> = (0..self.shared_state.entities().len())
                .into_par_iter()
                .flat_map(|i| self.animate_entity(EntityIndex(i)))
                .collect();
            messages.extend(entity_messages);
        }

        // Entities updates
        if self.local_state.get_frame_i() % self.config.entity_update_freq() == 0 {
            let entity_messages: Vec<Message> = (0..self.shared_state.entities().len())
                .into_par_iter()
                .flat_map(|i| self.update_entity(EntityIndex(i)))
                .collect();
            messages.extend(entity_messages);
        }

        messages
    }

    pub fn entity_is_squad_leader(&self, entity_i: EntityIndex) -> bool {
        let entity = self.shared_state.entity(entity_i);
        let squad_uuid = entity.squad_uuid();
        let squad_composition = self.shared_state.squad(squad_uuid);
        let squad_leader = squad_composition.leader();
        squad_leader == entity_i
    }

    pub fn entity_can_take_order(&self, _entity_i: EntityIndex, _order: &Order) -> bool {
        // TODO : check if order can be applied (e.g. if entity is not panicking, under fire, etc.)
        true
    }

    pub fn take_order(&self, entity_i: EntityIndex, order: &Order) -> Vec<Message> {
        // TODO : behavior must be given to other squad soldiers !!!! other soldiers must can accept it too (under fire etc)
        let mut messages = vec![];
        let entity = self.shared_state.entity(entity_i);

        messages.push(Message::SharedState(SharedStateMessage::Entity(
            entity_i,
            EntityMessage::SetBehavior(order.to_behavior()),
        )));

        messages
    }

    pub fn squad_leader_propagate_order(
        &self,
        squad_uuid: SquadUuid,
        order: &Order,
    ) -> Vec<Message> {
        let mut messages = vec![];
        let squad = self.shared_state.squad(squad_uuid);

        match order {
            Order::MoveTo(_) | Order::MoveFastTo(_) | Order::SneakTo(_) => {
                for (entity_index, point) in
                    squad_positions(squad, Formation::Line, &self.shared_state)
                {
                    let entity = self.shared_state.entity(entity_index);
                    if let Some(grid_path) = find_path(
                        &self.map,
                        &self.grid_point_from_world_point(entity.get_world_point()),
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
                            entity_index,
                            member_order,
                        )))
                    }
                }
            }
            Order::Defend(_) => todo!(),
            Order::Hide(_) => todo!(),
        }

        messages
    }
}
