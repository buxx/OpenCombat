use std::mem::discriminant;

use super::Engine;
use crate::{
    behavior::{Behavior, BehaviorMode},
    message::{Message, SharedStateMessage, SoldierMessage},
    order::Order,
    types::{Angle, SoldierIndex},
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

    // FIXME BS NOW : what about all times computed (network, change regulary, etc) ?
    pub fn resolve_soldier_behavior(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        let mut new_behavior: Option<Behavior> = None;
        let soldier = self.shared_state.soldier(soldier_index);
        let soldier_behavior = soldier.get_behavior();
        let soldier_order = soldier.order();

        if soldier.under_fire().is_max() {
            // TODO : soldier angle
            new_behavior = Some(Behavior::Hide(Angle(0.)));
        }

        // TODO : algorithm or config for values ?
        let order = soldier.order();

        match order {
            Order::MoveTo(path) => {
                if soldier.under_fire().is_warning() || soldier.under_fire().is_danger() {
                    new_behavior = Some(Behavior::SneakTo(path.clone()));
                }
            }
            Order::MoveFastTo(path) => {
                if soldier.under_fire().is_danger() {
                    new_behavior = Some(Behavior::SneakTo(path.clone()));
                }
            }
            Order::SneakTo(_) => {}
            Order::Defend(_) => {}
            Order::Hide(_) => {}
            Order::Idle => {
                if soldier.under_fire().value() > 0 {
                    // TODO : soldier angle
                    new_behavior = Some(Behavior::Hide(Angle(0.)));
                }
            }
        }

        // Behavior has been adapted according to context
        if let Some(new_behavior) = new_behavior {
            // And it is a new behavior (permit to not send it over network for nothing)
            if discriminant(&new_behavior) != discriminant(soldier_behavior) {
                return vec![Message::SharedState(SharedStateMessage::Soldier(
                    soldier_index,
                    SoldierMessage::SetBehavior(new_behavior),
                ))];
            }
        // If there is no adaptation
        } else if !soldier_behavior.match_with_order(order) {
            // Adapt soldier from order
            return self.take_order(soldier_index, order);
        }

        vec![]
    }
}
