use crate::{behavior::BehaviorMode, types::SoldierIndex};

use super::Engine;

impl Engine {
    pub fn soldier_behavior_mode(&self, soldier_index: SoldierIndex) -> BehaviorMode {
        if self.shared_state.soldier_board(soldier_index).is_some() {
            return BehaviorMode::Vehicle;
        }
        BehaviorMode::Ground
    }
}
