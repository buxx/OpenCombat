use crate::{message::*, types::*};

use super::Engine;

impl Engine {
    ///  - Compute visibility with other entities
    ///  - Compute behavior against physics (explosions, gunfires, ...)
    pub fn animate_entity(&self, entity_index: EntityIndex) -> Vec<Message> {
        let entity = self.shared_state.entity(entity_index);
        let mut messages = vec![];

        // Take new order if possible
        if self.entity_is_squad_leader(entity_index) {
            if let Some(order) = self.shared_state.order_for(entity_index) {
                if self.entity_can_take_order(entity_index, order) {
                    messages.extend(self.take_order(entity_index, order));
                    messages.push(Message::State(StateMessage::RemoveOder(
                        entity.squad_uuid(),
                    )));
                }
            }
        }

        messages
    }
}
