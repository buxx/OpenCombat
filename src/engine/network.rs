use crate::{
    message::{Message, NetworkMessage},
    sync::StateCopy,
};

use super::Engine;

impl Engine {
    /// Retrieve server or clients messages.
    pub fn sync(&self) -> Vec<Message> {
        self.network.incoming_messages()
    }

    pub fn dispatch(&self, message: &Message) {
        if match self.config.network_mode() {
            crate::NetWorkMode::Server => match message {
                // Server must broadcast all related entity messages to permit clients to update their states
                Message::Entity(_, _) => true,
                // Server do not broadcast this message but send state in response
                Message::Network(NetworkMessage::RequireCompleteSync) => false,
                // These messages are nevers reacted by server
                Message::Network(NetworkMessage::InitializeStateFrom(_)) => unreachable!(),
                Message::Network(NetworkMessage::Acknowledge) => unreachable!(),
            },
            crate::NetWorkMode::Client => match message {
                // Client do not dispatch entity messages because only consume them
                Message::Entity(_, _) => false,
                // Client do not dispatch initialization because only consume them
                Message::Network(NetworkMessage::InitializeStateFrom(_)) => false,
                // These messages are never reacted by client
                Message::Network(NetworkMessage::RequireCompleteSync) => unreachable!(),
                Message::Network(NetworkMessage::Acknowledge) => unreachable!(),
            },
        } {
            self.network.send(message.clone())
        }
    }
}
