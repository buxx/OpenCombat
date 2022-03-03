use crate::message::{EntityMessage, Message};

use super::MainState;

impl MainState {
    pub fn react(&mut self, messages: Vec<Message>) {
        for message in messages {
            match message {
                Message::Entity(entity_i, entity_message) => {
                    let entity = &mut self.entities[entity_i];
                    match entity_message {
                        EntityMessage::UpdateWorldPosition(new_world_position) => {
                            entity.set_world_position(new_world_position)
                        }
                    }
                }
            }
        }
    }
}
