use crate::{
    message::{Message, SharedStateMessage, SoldierMessage},
    types::SoldierIndex,
};

use super::Engine;

impl Engine {
    pub fn decrease_feeling(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        vec![
            Message::SharedState(SharedStateMessage::Soldier(
                soldier_index,
                SoldierMessage::DecreaseUnderFire,
            )),
            Message::SharedState(SharedStateMessage::Soldier(
                soldier_index,
                SoldierMessage::DecreaseFear,
            )),
        ]
    }
}
