use crate::{
    behavior::{feeling::Feeling, Behavior},
    message::{Message, SharedStateMessage, SoldierMessage},
    types::{Meters, SoldierIndex},
};

use super::Engine;

impl Engine {
    pub fn soldier_blast_stunned(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        vec![
            Message::SharedState(SharedStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetBehavior(Behavior::Unconscious),
            )),
            Message::SharedState(SharedStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetUnconscious(true),
            )),
        ]
    }

    // TODO : have a real algorithm here
    pub fn soldier_blast(&self, soldier_index: SoldierIndex, distance: Meters) -> Vec<Message> {
        vec![Message::SharedState(SharedStateMessage::Soldier(
            soldier_index,
            SoldierMessage::IncreaseUnderFire(Feeling::UnderFire(0).blast_increase_value(distance)),
        ))]
    }
}
