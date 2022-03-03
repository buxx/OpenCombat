use crate::message::Message;

use super::Engine;

impl Engine {
    pub fn react(&mut self, messages: Vec<Message>) {
        for message in messages {
            match message {
                Message::Entity(entity_i, entity_message) => {
                    self.state.react_entity_message(entity_i, entity_message);
                }
            }
        }
    }
}
