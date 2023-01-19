use super::shared::SharedState;
use crate::{
    message::{SideEffect, SoldierMessage},
    types::*,
};

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
            SoldierMessage::SetGesture(gesture) => {
                soldier.set_gesture(gesture);
            }
            SoldierMessage::SetOrientation(angle) => soldier.set_looking_direction(angle),
            SoldierMessage::ReachBehaviorStep => {
                if soldier.order_mut().reach_step() || soldier.get_behavior_mut().reach_step() {
                    return vec![SideEffect::SoldierFinishHisBehavior(soldier_index)];
                }
            }
            SoldierMessage::SetAlive(alive) => soldier.set_alive(alive),
            SoldierMessage::SetUnconscious(unconscious) => soldier.set_unconscious(unconscious),
            SoldierMessage::IncreaseUnderFire(value) => soldier.increase_under_fire(value),
            SoldierMessage::DecreaseUnderFire => soldier.decrease_under_fire(),
            SoldierMessage::SetOrder(order) => soldier.set_order(order),
            SoldierMessage::ReloadWeapon(class) => soldier.reload_weapon(&class),
            SoldierMessage::WeaponShot(class) => soldier.weapon_shot(&class),
        }

        vec![]
    }
}
