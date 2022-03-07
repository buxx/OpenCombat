use crate::{
    behavior::{walking, Behavior},
    message::*,
    types::*,
};

use super::Engine;

impl Engine {
    ///  - World pixel position according to movement
    ///  - ...
    pub fn update_entity(&self, i: EntityIndex) -> Vec<Message> {
        let entity = self.state.entity(i);
        let mut messages = vec![];

        match entity.get_behavior() {
            Behavior::Idle => {}
            Behavior::WalkingTo(destination) => {
                messages.extend(Message::vec_from_entity(
                    i,
                    walking::entity_updates(entity, destination),
                ));
            }
        }

        messages
    }
}
