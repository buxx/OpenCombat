use crate::{
    behavior::{walking, Behavior},
    message::*,
    types::*,
};

use super::Engine;

impl Engine {
    ///  - World pixel point according to movement
    ///  - ...
    pub fn update_entity(&self, i: EntityIndex) -> Vec<Message> {
        let entity = self.state.entity(i);
        let mut messages = vec![];

        let entity_messages = match entity.get_behavior() {
            Behavior::Idle => {
                vec![]
            }
            Behavior::WalkingTo(path) => walking::entity_updates(entity, path),
        };
        messages.extend(Message::vec_from_entity(i, entity_messages));

        messages
    }
}
