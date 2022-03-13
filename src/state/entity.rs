use super::shared::SharedState;
use crate::{message::EntityMessage, types::*};

impl SharedState {
    pub fn react_entity_message(&mut self, entity_i: EntityIndex, entity_message: EntityMessage) {
        if !self.initialized() {
            return;
        }

        let entity = &mut self.entity_mut(entity_i);
        match entity_message {
            EntityMessage::SetWorldPosition(new_world_point) => {
                entity.set_world_point(new_world_point)
            }
            EntityMessage::SetBehavior(behavior) => entity.set_behavior(behavior),
            EntityMessage::SetOrientation(angle) => entity.set_looking_direction(angle),
            EntityMessage::ReachBehaviorStep => entity.get_behavior_mut().reach_step(),
        }
    }
}
