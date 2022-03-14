use crate::{message::*, sync::StateCopy};

use super::Engine;

impl Engine {
    pub fn react(&mut self, messages: Vec<Message>) {
        // Dispatch messages depend on network mode
        match self.config.network_mode() {
            crate::NetWorkMode::Server => self.dispatch_as_server(&messages),
            crate::NetWorkMode::Client => self.dispatch_as_client(&messages),
        }

        for message in messages {
            match message {
                Message::SharedState(shared_state_message) => {
                    self.shared_state.react(shared_state_message)
                }
                Message::LocalState(local_state_message) => {
                    self.local_state.react(local_state_message)
                }
                Message::Network(network_message) => match network_message {
                    NetworkMessage::RequireCompleteSync => {
                        self.send_complete_sync();
                    }
                    NetworkMessage::InitializeStateFrom(state_copy) => {
                        self.shared_state.init_from_copy(state_copy);
                    }
                    NetworkMessage::Acknowledge => unreachable!(),
                },
            }
        }
    }

    fn send_complete_sync(&self) {
        let state_copy = StateCopy::from_state(&self.shared_state);
        let network_message = NetworkMessage::InitializeStateFrom(state_copy);
        let message = Message::Network(network_message);
        self.network.send(vec![message]);
    }
}
