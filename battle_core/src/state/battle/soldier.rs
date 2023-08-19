use crate::{behavior::BehaviorMode, entity::soldier::Soldier, types::SoldierIndex};

use super::{
    message::{SideEffect, SoldierMessage},
    BattleState,
};

impl BattleState {
    pub fn react_soldier_message(
        &mut self,
        soldier_index: &SoldierIndex,
        soldier_message: &SoldierMessage,
    ) -> Vec<SideEffect> {
        let soldier = &mut self.soldier_mut(*soldier_index);
        match soldier_message {
            SoldierMessage::SetWorldPosition(new_world_point) => {
                soldier.set_world_point(*new_world_point)
            }
            SoldierMessage::SetBehavior(behavior) => {
                soldier.set_behavior(behavior.clone());
                return vec![SideEffect::RefreshEntityAnimation(*soldier_index)];
            }
            SoldierMessage::SetGesture(gesture) => {
                soldier.set_gesture(gesture.clone());
            }
            SoldierMessage::SetOrientation(angle) => soldier.set_looking_direction(*angle),
            SoldierMessage::ReachBehaviorStep => {
                if soldier.order_mut().reach_step() || soldier.behavior_mut().reach_step() {
                    return vec![SideEffect::SoldierFinishHisBehavior(
                        *soldier_index,
                        soldier.order().then().clone(),
                    )];
                }
            }
            SoldierMessage::SetAlive(alive) => soldier.set_alive(*alive),
            SoldierMessage::SetUnconscious(unconscious) => soldier.set_unconscious(*unconscious),
            SoldierMessage::IncreaseUnderFire(value) => soldier.increase_under_fire(*value),
            SoldierMessage::DecreaseUnderFire => soldier.decrease_under_fire(),
            SoldierMessage::SetOrder(order) => soldier.set_order(order.clone()),
            SoldierMessage::ReloadWeapon(class) => soldier.reload_weapon(&class),
            SoldierMessage::WeaponShot(class) => soldier.weapon_shot(&class),
            SoldierMessage::SetLastShootFrameI(frame_i) => soldier.set_last_shoot_frame_i(*frame_i),
        }

        vec![]
    }

    pub fn soldier_behavior_mode(&self, soldier: &Soldier) -> BehaviorMode {
        if self.soldier_board(soldier.uuid()).is_some() {
            return BehaviorMode::Vehicle;
        }
        BehaviorMode::Ground
    }
}
