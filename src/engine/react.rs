use crate::{message::*, state::SideEffect, sync::StateCopy};

use super::Engine;

impl Engine {
    pub fn react(&mut self, messages: Vec<Message>) -> Vec<SideEffect> {
        let mut side_effects = vec![];

        for message in messages {
            match message {
                Message::SharedState(shared_state_message) => {
                    side_effects.extend(self.shared_state.react(shared_state_message));
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
                        self.graphics.initialize(self.shared_state.entities());
                    }
                    NetworkMessage::Acknowledge => unreachable!(),
                },
            }
        }

        side_effects
    }

    fn send_complete_sync(&self) {
        let state_copy = StateCopy::from_state(&self.shared_state);
        let network_message = NetworkMessage::InitializeStateFrom(state_copy);
        let message = Message::Network(network_message);
        self.network.send(vec![message]);
    }
}
