use crate::{
    message::{Message, NetworkMessage},
    sync::StateCopy,
};

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
                Message::Network(network_message) => match network_message {
                    NetworkMessage::RequireCompleteSync => {
                        self.network
                            .send(Message::Network(NetworkMessage::InitializeStateFrom(
                                StateCopy::from_state(&self.state),
                            )));
                    }
                    NetworkMessage::InitializeStateFrom(state_copy) => {
                        println!("{:?} : InitializeStateFrom", self.config.network_mode());
                        self.state.init_from_copy(state_copy);
                    }
                    NetworkMessage::Acknowledge => unreachable!(),
                },
            }
        }
    }
}
