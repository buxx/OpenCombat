use super::{shared::SharedState, SideEffect};
use crate::{message::EntityMessage, types::*};

impl SharedState {
    pub fn react_entity_message(
        &mut self,
        entity_i: EntityIndex,
        entity_message: EntityMessage,
    ) -> Vec<SideEffect> {
        if !self.initialized() {
            return vec![];
        }

        let entity = &mut self.entity_mut(entity_i);
        match entity_message {
            EntityMessage::SetWorldPosition(new_world_point) => {
                entity.set_world_point(new_world_point)
            }
            EntityMessage::SetBehavior(behavior) => {
                entity.set_behavior(behavior);
                return vec![SideEffect::RefreshEntityAnimation(entity_i)];
            }
            EntityMessage::SetOrientation(angle) => entity.set_looking_direction(angle),
            EntityMessage::ReachBehaviorStep => entity.get_behavior_mut().reach_step(),
        }

        vec![]
    }
}
