use crate::message::Message;

use super::Engine;

impl Engine {
    pub fn react(&mut self, messages: Vec<Message>) {
        for message in messages {
            // Route to server or client depending on message type.
            self.dispatch(&message);

            match message {
                Message::Entity(entity_i, entity_message) => {
                    self.state.react_entity_message(entity_i, entity_message);
                }
                Message::Foo => {}
                Message::Network(network_message) => match self.config.network_mode() {
                    crate::NetWorkMode::Server => self.react_network_message(network_message),
                    crate::NetWorkMode::Client => {}
                },
            }
        }
    }
}
