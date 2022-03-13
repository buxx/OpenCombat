use crate::{message::*, order::Order, types::*};
use rayon::prelude::*;

use super::Engine;

impl Engine {
    pub fn tick_entities(&self) -> Vec<Message> {
        let mut messages = vec![];

        // Entities animation
        if self.frame_i % self.config.entity_animate_freq() == 0 {
            let entity_messages: Vec<Message> = (0..self.shared_state.entities().len())
                .into_par_iter()
                .flat_map(|i| self.animate_entity(EntityIndex(i)))
                .collect();
            messages.extend(entity_messages);
        }

        // Entities updates
        if self.frame_i % self.config.entity_update_freq() == 0 {
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
        let behavior = order.to_behavior();
        let entity_message = EntityMessage::SetBehavior(behavior);
        vec![Message::State(StateMessage::Entity(
            entity_i,
            entity_message,
        ))]
    }
}
