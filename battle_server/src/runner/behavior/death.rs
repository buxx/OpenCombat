use battle_core::{
    behavior::Behavior,
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::SoldierIndex,
};

use crate::runner::{message::RunnerMessage, Runner};

impl Runner {
    pub fn soldier_die(&self, soldier_index: SoldierIndex) -> Vec<RunnerMessage> {
        vec![
            RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetBehavior(Behavior::Dead),
            )),
            RunnerMessage::BattleState(BattleStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetAlive(false),
            )),
        ]
    }
}
