use crate::message::{Message, SharedStateMessage, SoldierMessage};

use super::Engine;

impl Engine {
    /// Retrieve server or clients messages.
    pub fn sync(&self) -> Vec<Message> {
        puffin::profile_scope!("sync");
        self.network.incoming_messages()
    }

    pub fn dispatch_as_server(&self, messages: &Vec<Message>) {
        puffin::profile_scope!("dispatch_as_server");
        let mut dispatch_messages: Vec<Message> = vec![];

        for message in messages {
            match message {
                // State changes must be sent to clients
                Message::SharedState(_) => dispatch_messages.push(message.clone()),
                // Physics are displayed by clients and computed by server
                Message::Physics(_) => dispatch_messages.push(message.clone()),
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
                // New order
                Message::SharedState(SharedStateMessage::Soldier(
                    _,
                    SoldierMessage::SetOrder(_),
                )) => dispatch_messages.push(message.clone()),
                _ => {}
            }
        }

        // Send messages by group to avoid zmq queue overflow
        self.network.send(dispatch_messages);
    }

    pub fn deal_with_sync_errors_as_server(&self) -> Vec<Message> {
        puffin::profile_scope!("deal_with_sync_errors_as_server");
        let messages: Vec<Message> = vec![];

        for error in self.network.errors() {
            // For now, only print errors
            println!("ERROR :: Network error :: {}", error)
        }

        messages
    }

    pub fn deal_with_sync_errors_as_client(&self) -> Vec<Message> {
        let messages: Vec<Message> = vec![];

        for error in self.network.errors() {
            // For now, only print errors
            println!("ERROR :: Network error :: {}", error)
        }

        messages
    }
}
