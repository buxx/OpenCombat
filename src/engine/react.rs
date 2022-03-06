use crate::message::Message;

use super::Engine;

impl Engine {
    pub fn react(&mut self, messages: Vec<Message>) {
        for message in messages {
            // Route to server or client depending on message type.
            self.dispatch(&message);

            match message {
                Message::Entity(entity_i, entity_message) => {
                    // Apply messages on entities
                    self.state.react_entity_message(entity_i, entity_message);
                }
                Message::Network(network_message) => {
                    //
                    self.react_network_message(network_message)
                }
            }
        }
    }
}
