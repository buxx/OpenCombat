use crate::{behavior::Behavior, message::*, types::*};

use super::Engine;

impl Engine {
    pub fn movement_updates(&self, entity_index: EntityIndex, path: &WorldPaths) -> Vec<Message> {
        let mut messages = vec![];
        let entity = self.shared_state.entity(entity_index);

        let point = path.next_point().expect("Must have point in path");

        // There is a next point in path, go to it
        let velocity = entity
            .get_behavior()
            .velocity()
            .expect("Entity behavior must have velocity when move code called");
        let vector = (point.to_vec2() - entity.get_world_point().to_vec2()).normalize() * velocity;
        // Point reached
        if (entity.get_world_point().to_vec2() - point.to_vec2()).length() <= vector.length() {
            // If it is the last point, move is finished
            if path.is_last_point().expect("Must contain points") {
                messages.push(Message::SharedState(SharedStateMessage::Entity(
                    entity_index,
                    EntityMessage::SetBehavior(Behavior::Idle),
                )));
            } else {
                messages.push(Message::SharedState(SharedStateMessage::Entity(
                    entity_index,
                    EntityMessage::ReachBehaviorStep,
                )));
            }

            // If this is a squad leader, propagate current order

            if self.entity_is_squad_leader(entity_index) {
                // Send order to other squad members
                if let Some(order) = entity.get_behavior().to_order() {
                    messages.extend(self.squad_leader_propagate_order(entity.squad_uuid(), &order));
                }
            };

            // Movement required
        } else {
            let new_point = entity.get_world_point().apply(vector);
            messages.push(Message::SharedState(SharedStateMessage::Entity(
                entity_index,
                EntityMessage::SetWorldPosition(new_point),
            )));
        }

        messages
    }
}
