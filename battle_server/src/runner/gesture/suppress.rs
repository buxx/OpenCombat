use battle_core::{
    behavior::gesture::{Gesture, GestureContext},
    entity::soldier::Soldier,
    types::WorldPoint,
};

use crate::runner::Runner;

impl Runner {
    pub fn suppress_fire_gesture(
        &self,
        soldier: &Soldier,
        point: &WorldPoint,
    ) -> (GestureContext, Gesture) {
        if let Some(weapon) = self.soldier_able_to_fire_on_point(soldier, point) {
            return self.engage_point_gesture(soldier, point, weapon);
        }

        (GestureContext::Idle, Gesture::Idle)
    }
}
