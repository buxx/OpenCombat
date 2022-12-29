use crate::message::{Message, SharedStateMessage};

use super::Engine;

impl Engine {
    /// Retrieve server or clients messages.
    pub fn sync(&self) -> Vec<Message> {
        self.network.incoming_messages()
    }

    pub fn dispatch_as_server(&self, messages: &Vec<Message>) {
        let mut dispatch_messages: Vec<Message> = vec![];

        for message in messages {
            match message {
                // State changes must be sent to clients
                Message::SharedState(_) => dispatch_messages.push(message.clone()),
                _ => {}
            }
        }

        // Send messages by group to avoid zmq queue overflow
        self.network.send(dispatch_messages);
    }

    pub fn dispatch_as_client(&self, messages: &Vec<Message>) {
        let mut dispatch_messages: Vec<Message> = vec![];

        for message in messages {
            match message {
                // State changes must be sent to clients
                Message::SharedState(SharedStateMessage::PushCommandOrder(_, _))
                | Message::SharedState(SharedStateMessage::PushSquadOrder(_, _))
                | Message::SharedState(SharedStateMessage::PushBulletFire(_))
                | Message::SharedState(SharedStateMessage::PushExplosion(_)) => {
                    dispatch_messages.push(message.clone())
                }
                _ => {}
            }
        }

        // Send messages by group to avoid zmq queue overflow
        self.network.send(dispatch_messages);
    }

    pub fn deal_with_sync_errors_as_server(&self) -> Vec<Message> {
        let messages: Vec<Message> = vec![];

        for error in self.network.errors() {
            // For now, only print errors
            println!("Network error : {}", error)
        }

        messages
    }

    pub fn deal_with_sync_errors_as_client(&self) -> Vec<Message> {
        let messages: Vec<Message> = vec![];

        for error in self.network.errors() {
            // For now, only print errors
            println!("Network error : {}", error)
        }

        messages
    }
}
