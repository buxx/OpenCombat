use battle_core::{entity::soldier::Soldier, types::WorldPoint};

use crate::runner::Runner;

use super::GestureResult;

impl Runner {
    pub fn suppress_fire_gesture(&self, soldier: &Soldier, point: &WorldPoint) -> GestureResult {
        if let Some(weapon) = self.soldier_able_to_fire_on_point(soldier, point) {
            let (gesture_context, gesture) = self.engage_point_gesture(soldier, weapon);
            return GestureResult::Handled(gesture_context, gesture);
        }

        GestureResult::Cant(None)
    }
}
