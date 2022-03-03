use crate::{message::Message, types::*};

use super::Orchestrator;

impl Orchestrator {
    pub fn tick_entity(&self, i: EntityIndex) -> Vec<Message> {
        vec![]
    }
}
