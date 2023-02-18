use battle_core::{
    behavior::gesture::{Gesture, GestureContext},
    entity::soldier::Soldier,
    types::SoldierIndex,
};

use crate::runner::Runner;

impl Runner {
    pub fn engage_soldier_gesture(
        &self,
        soldier: &Soldier,
        engaged_soldier_index: &SoldierIndex,
    ) -> (GestureContext, Gesture) {
        let point = self
            .battle_state
            .soldier(*engaged_soldier_index)
            .get_world_point();

        if let Some(weapon) = self.soldier_able_to_fire_on_point(soldier, &point) {
            return self.engage_point_gesture(soldier, &point, weapon);
        }

        (GestureContext::Idle, Gesture::Idle)
    }
}
