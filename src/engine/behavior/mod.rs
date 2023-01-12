use std::mem::discriminant;

use super::Engine;
use crate::{
    behavior::{Behavior, BehaviorMode},
    entity::soldier::Soldier,
    message::{Message, SharedStateMessage, SoldierMessage},
    order::Order,
    types::{Angle, SoldierIndex, WorldPaths},
};

mod blast;
mod bullet;
mod death;

impl Engine {
    pub fn soldier_behavior_mode(&self, soldier_index: SoldierIndex) -> BehaviorMode {
        if self.shared_state.soldier_board(soldier_index).is_some() {
            return BehaviorMode::Vehicle;
        }
        BehaviorMode::Ground
    }

    pub fn resolve_soldier_behavior(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        let soldier = self.shared_state.soldier(soldier_index);
        let order = soldier.order();
        let behavior = match order {
            Order::Idle => self.resolve_soldier_idle_behavior(soldier),
            Order::MoveTo(paths) => self.resolve_soldier_move_behavior(soldier, paths),
            Order::MoveFastTo(paths) => self.resolve_soldier_move_behavior(soldier, paths),
            Order::SneakTo(paths) => self.resolve_soldier_move_fast_behavior(soldier, paths),
            Order::Defend(angle) => self.resolve_soldier_defend_behavior(soldier, angle),
            Order::Hide(angle) => self.resolve_soldier_hide_behavior(soldier, angle),
        };

        // Change soldier behavior if computed behavior is different from its current behavior
        if &behavior != soldier.get_behavior() {
            return vec![Message::SharedState(SharedStateMessage::Soldier(
                soldier_index,
                SoldierMessage::SetBehavior(behavior),
            ))];
        };

        vec![]
    }

    pub fn resolve_soldier_idle_behavior(&self, soldier: &Soldier) -> Behavior {
        if soldier.under_fire().exist() {
            // TODO : soldier angle
            Behavior::Hide(Angle(0.))
        } else {
            Behavior::Idle
        }
    }

    pub fn resolve_soldier_move_behavior(&self, soldier: &Soldier, paths: &WorldPaths) -> Behavior {
        if soldier.under_fire().is_warning()
            || soldier.under_fire().is_danger()
            || soldier.under_fire().is_max()
        {
            Behavior::SneakTo(paths.clone())
        } else {
            Behavior::MoveTo(paths.clone())
        }
    }

    pub fn resolve_soldier_move_fast_behavior(
        &self,
        soldier: &Soldier,
        paths: &WorldPaths,
    ) -> Behavior {
        if soldier.under_fire().is_danger() || soldier.under_fire().is_max() {
            Behavior::SneakTo(paths.clone())
        } else {
            Behavior::MoveFastTo(paths.clone())
        }
    }

    pub fn resolve_soldier_defend_behavior(&self, _soldier: &Soldier, angle: &Angle) -> Behavior {
        Behavior::Defend(*angle)
    }

    pub fn resolve_soldier_hide_behavior(&self, _soldier: &Soldier, angle: &Angle) -> Behavior {
        Behavior::Hide(*angle)
    }
}
