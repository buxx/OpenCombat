use crate::message::Message;

use super::Orchestrator;

impl Orchestrator {
    pub fn react(&mut self, messages: Vec<Message>) {
        for message in messages {
            println!("{:?}", message);
        }
    }
}
