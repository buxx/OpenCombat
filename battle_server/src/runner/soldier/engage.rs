use battle_core::{
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::SoldierIndex,
    utils::angle,
};

use crate::runner::message::RunnerMessage;

use super::SoldierRunner;

impl SoldierRunner {
    pub fn engage_update(
        &self,
        soldier_index: &SoldierIndex,
        target_index: &SoldierIndex,
    ) -> Vec<RunnerMessage> {
        let soldier = self.battle_state().soldier(*soldier_index);
        let target = self.battle_state().soldier(*target_index);
        let angle = angle(&target.world_point(), &soldier.world_point());
        vec![RunnerMessage::BattleState(BattleStateMessage::Soldier(
            *soldier_index,
            SoldierMessage::SetOrientation(angle),
        ))]
    }
}
