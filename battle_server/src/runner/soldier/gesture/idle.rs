use battle_core::{
    behavior::{
        gesture::{Gesture, GestureContext},
        Behavior,
    },
    entity::soldier::Soldier,
};

use crate::runner::soldier::{fight::ChooseMethod, SoldierRunner};

use super::{FallbackBehavior, GestureResult};

impl SoldierRunner {
    pub fn idle_gesture(&self, soldier: &Soldier) -> GestureResult {
        if let Some(opponent) =
            self.soldier_find_opponent_to_target(soldier, None, &ChooseMethod::RandomFromNearest)
        {
            let point = opponent.world_point();
            if self
                .soldier_able_to_fire_on_point(soldier, &point)
                .is_some()
            {
                return GestureResult::Cant(Some(FallbackBehavior(Behavior::EngageSoldier(
                    opponent.uuid(),
                ))));
            }
        }

        GestureResult::Handled(GestureContext::Idle, Gesture::Idle)
    }
}
