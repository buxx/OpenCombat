use battle_core::{
    behavior::{feeling::Feeling, Behavior},
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::{Distance, SoldierIndex},
};

use crate::runner::{message::RunnerMessage, Runner};

impl Runner {
    pub fn soldier_blast_stunned(&self, soldier_index: SoldierIndex) -> Vec<RunnerMessage> {
        vec![
            RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetBehavior(Behavior::Unconscious),
            )),
            RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetUnconscious(true),
            )),
        ]
    }

    // TODO : have a real algorithm here
    pub fn soldier_blast(
        &self,
        soldier_index: SoldierIndex,
        distance: Distance,
    ) -> Vec<RunnerMessage> {
        vec![RunnerMessage::BattleState(BattleStateMessage::Soldier(
            soldier_index,
            SoldierMessage::IncreaseUnderFire(Feeling::blast_increase_value(distance)),
        ))]
    }
}
