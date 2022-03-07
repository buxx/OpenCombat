use crate::{
    message::{Message, NetworkMessage},
    sync::StateCopy,
};

use super::Engine;

impl Engine {
    pub fn react(&mut self, messages: Vec<Message>) {
        // Route to server or client depending on message type.
        self.dispatch(&messages);

        for message in messages {
            match message {
                Message::Entity(entity_i, entity_message) => {
                    self.state.react_entity_message(entity_i, entity_message);
                }
                Message::Network(network_message) => match network_message {
                    NetworkMessage::RequireCompleteSync => {
                        let state_copy = StateCopy::from_state(&self.state);
                        let network_message = NetworkMessage::InitializeStateFrom(state_copy);
                        let message = Message::Network(network_message);
                        self.network.send(vec![message]);
                    }
                    NetworkMessage::InitializeStateFrom(state_copy) => {
                        self.state.init_from_copy(state_copy);
                    }
                    NetworkMessage::Acknowledge => unreachable!(),
                },
            }
        }
    }
}
