use crate::message::{EntityMessage, Message};

use super::Engine;

impl Engine {
    pub fn react(&mut self, messages: Vec<Message>) {
        for message in messages {
            match message {
                Message::Entity(entity_i, entity_message) => {
                    self.react_entity_message(entity_i, entity_message);
                }
            }
        }
    }

    fn react_entity_message(&mut self, entity_i: usize, entity_message: EntityMessage) {
        let entity = &mut self.entities[entity_i];
        match entity_message {
            EntityMessage::UpdateWorldPosition(new_world_position) => {
                entity.set_world_position(new_world_position)
            }
        }
    }
}
