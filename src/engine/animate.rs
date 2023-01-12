use super::Engine;
use crate::{
    message::{Message, SharedStateMessage},
    types::SoldierIndex,
};

impl Engine {
    ///  - Compute visibility with other soldiers
    ///  - Compute behavior against physics (explosions, gunfires, ...)
    pub fn animate_soldier(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        let soldier = self.shared_state.soldier(soldier_index);
        if !soldier.can_be_animated() {
            return vec![];
        }

        let mut messages = vec![];

        // Take new order for squad leader
        if self.soldier_is_squad_leader(soldier_index) {
            if let Some(order) = self.shared_state.order_for_squad_leader(soldier_index) {
                if order != soldier.order() {
                    messages.extend(self.take_order(soldier_index, order));
                    messages.push(Message::SharedState(SharedStateMessage::RemoveCommandOder(
                        soldier.squad_uuid(),
                    )));
                }
            }

        // Take new order for squad member
        } else {
            if let Some(order) = self.shared_state.order_for_squad_member(soldier_index) {
                if order != soldier.order() {
                    messages.extend(self.take_order(soldier_index, order));
                    messages.push(Message::SharedState(SharedStateMessage::RemoveSquadOder(
                        soldier_index,
                    )));
                }
            }
        }

        // Adapt behavior according to feelings
        messages.extend(self.resolve_soldier_behavior(soldier_index));

        messages
    }
}
