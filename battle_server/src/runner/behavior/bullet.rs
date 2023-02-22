use battle_core::{
    behavior::feeling::Feeling,
    state::battle::message::{BattleStateMessage, SoldierMessage},
    types::{Distance, SoldierIndex},
};

use crate::runner::{message::RunnerMessage, Runner};

impl Runner {
    // TODO : have a real algorithm here
    pub fn soldier_bullet_injured(&self, _soldier_index: SoldierIndex) -> Vec<RunnerMessage> {
        vec![]
    }

    // TODO : have a real algorithm here
    pub fn soldier_proximity_bullet(
        &self,
        soldier_index: SoldierIndex,
        distance: Distance,
    ) -> Vec<RunnerMessage> {
        vec![RunnerMessage::BattleState(BattleStateMessage::Soldier(
            soldier_index,
            SoldierMessage::IncreaseUnderFire(
                Feeling::UnderFire(0).proximity_bullet_increase_value(distance),
            ),
        ))]
    }
}
