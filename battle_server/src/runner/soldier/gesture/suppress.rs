use battle_core::{entity::soldier::Soldier, types::WorldPoint};

use crate::runner::soldier::SoldierRunner;

use super::GestureResult;

impl SoldierRunner {
    pub fn suppress_fire_gesture(&self, soldier: &Soldier, point: &WorldPoint) -> GestureResult {
        if let Some(weapon) = self.soldier_able_to_fire_on_point(soldier, point) {
            let (gesture_context, gesture) = self.engage_point_gesture(soldier, point, weapon);
            return GestureResult::Handled(gesture_context, gesture);
        }

        GestureResult::Cant(None)
    }
}
