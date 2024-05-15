use battle_core::{entity::soldier::Soldier, types::SoldierIndex};

use crate::runner::Runner;

use super::GestureResult;

impl Runner {
    pub fn engage_soldier_gesture(
        &self,
        soldier: &Soldier,
        engaged_soldier_index: &SoldierIndex,
    ) -> GestureResult {
        let target_soldier = self.battle_state.soldier(*engaged_soldier_index);

        if target_soldier.can_be_designed_as_target() {
            let target_soldier_point = target_soldier.world_point();

            if let Some(engagement) =
                self.soldier_able_to_fire_on_point(soldier, &target_soldier_point)
            {
                let (gesture_context, gesture) = self.engage_point_gesture(soldier, engagement);
                return GestureResult::Handled(gesture_context, gesture);
            }
        }

        GestureResult::Cant(None)
    }
}
