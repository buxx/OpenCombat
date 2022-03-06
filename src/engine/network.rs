use crate::message::{Message, NetworkMessage};

use super::Engine;

impl Engine {
    /// Retrieve server or clients messages.
    pub fn sync(&self) -> Vec<Message> {
        self.network.incoming_messages()
    }

    pub fn dispatch(&self, message: &Message) {
        match self.config.network_mode() {
            crate::NetWorkMode::Server => {
                if message.broadcast() {
                    // println!("Broadcast message: {:?}", message);
                    self.network.send(message.clone())
                }
            }
            crate::NetWorkMode::Client => {
                if message.sync() {
                    self.network.send(message.clone())
                }
            }
        }
    }

    pub fn react_network_message(&self, message: NetworkMessage) {
        match message {
            NetworkMessage::RequireCompleteSync => {
                println!("Require complete sync required");
            }
            NetworkMessage::Acknowledge => {}
        }
    }
}
