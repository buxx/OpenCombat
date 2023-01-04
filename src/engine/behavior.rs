use crate::{
    behavior::{feeling::Feeling, Behavior, BehaviorMode},
    config::TARGET_FPS,
    message::{Message, SharedStateMessage, SoldierMessage},
    types::{Meters, SoldierIndex},
};

use super::Engine;

impl Engine {
    pub fn soldier_behavior_mode(&self, soldier_index: SoldierIndex) -> BehaviorMode {
        if self.shared_state.soldier_board(soldier_index).is_some() {
            return BehaviorMode::Vehicle;
        }
        BehaviorMode::Ground
    }

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

    pub fn soldier_stunned(&self, soldier_index: SoldierIndex) -> Vec<Message> {
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
