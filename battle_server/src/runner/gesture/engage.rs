use battle_core::{behavior::Behavior, entity::soldier::Soldier, types::SoldierIndex};

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
            let point = target_soldier.world_point();
            if let Some(weapon) = self.soldier_able_to_fire_on_point(soldier, &point) {
                let (gesture_context, gesture) = self.engage_point_gesture(soldier, &point, weapon);
                return GestureResult::Handled(gesture_context, gesture);
            }
        }

        GestureResult::SwitchToBehavior(Behavior::Idle)
    }
}
