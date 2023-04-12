use battle_core::{
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::SoldierIndex,
    utils::angle,
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn engage_update(
        &self,
        soldier_index: &SoldierIndex,
        target_index: &SoldierIndex,
    ) -> Vec<RunnerMessage> {
        let soldier = self.battle_state.soldier(*soldier_index);
        let target = self.battle_state.soldier(*target_index);
        let angle = angle(&target.get_world_point(), &soldier.get_world_point());
        vec![RunnerMessage::BattleState(BattleStateMessage::Soldier(
            *soldier_index,
            SoldierMessage::SetOrientation(angle),
        ))]
    }
}
