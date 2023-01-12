use crate::{
    behavior::Behavior,
    message::{Message, SharedStateMessage, SoldierMessage},
    types::SoldierIndex,
};

use super::Engine;

impl Engine {
    pub fn soldier_die(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        vec![
            Message::SharedState(SharedStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetBehavior(Behavior::Dead),
            )),
            Message::SharedState(SharedStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetAlive(false),
            )),
        ]
    }
}
