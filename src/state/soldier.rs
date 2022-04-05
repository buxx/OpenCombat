use super::{shared::SharedState, SideEffect};
use crate::{message::SoldierMessage, types::*};

impl SharedState {
    pub fn react_soldier_message(
        &mut self,
        soldier_index: SoldierIndex,
        soldier_message: SoldierMessage,
    ) -> Vec<SideEffect> {
        if !self.initialized() {
            return vec![];
        }

        let soldier = &mut self.soldier_mut(soldier_index);
        match soldier_message {
            SoldierMessage::SetWorldPosition(new_world_point) => {
                soldier.set_world_point(new_world_point)
            }
            SoldierMessage::SetBehavior(behavior) => {
                soldier.set_behavior(behavior);
                return vec![SideEffect::RefreshEntityAnimation(soldier_index)];
            }
            SoldierMessage::SetOrientation(angle) => soldier.set_looking_direction(angle),
            SoldierMessage::ReachBehaviorStep => soldier.get_behavior_mut().reach_step(),
        }

        vec![]
    }
}