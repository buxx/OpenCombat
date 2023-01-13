use crate::{entity::soldier::Soldier, message::Message};

use super::Engine;

impl Engine {
    pub fn soldier_gesture(&self, soldier: &Soldier) -> Vec<Message> {
        vec![]
    }
}
