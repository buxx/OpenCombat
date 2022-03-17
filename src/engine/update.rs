use crate::{
    behavior::{move_, Behavior},
    message::*,
    types::*,
    utils::angle,
};

use super::Engine;

impl Engine {
    ///  - World pixel point according to movement
    ///  - ...
    pub fn update_entity(&self, i: EntityIndex) -> Vec<Message> {
        let mut messages = vec![];

        messages.extend(self.orientation_update(i));
        messages.extend(self.behavior_update(i));

        messages
    }

    fn orientation_update(&self, i: EntityIndex) -> Vec<Message> {
        let entity = self.shared_state.entity(i);
        let mut messages = vec![];

        if let Some(angle_) = entity.get_behavior().angle(entity.get_world_point()) {
            let entity_message = EntityMessage::SetOrientation(angle_);
            messages.push(Message::SharedState(SharedStateMessage::Entity(
                i,
                entity_message,
            )));
        }

        messages
    }

    fn behavior_update(&self, i: EntityIndex) -> Vec<Message> {
        let entity = self.shared_state.entity(i);
        let mut messages = vec![];

        let entity_messages = match entity.get_behavior() {
            Behavior::Idle => {
                vec![]
            }
            Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => {
                move_::entity_updates(entity, paths)
            }
            Behavior::Defend(_) => {
                vec![]
            }
            Behavior::Hide(_) => {
                vec![]
            }
        };
        messages.extend(Message::vec_from_entity(i, entity_messages));

        messages
    }
}
