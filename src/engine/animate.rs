use std::mem::discriminant;

use super::Engine;
use crate::{
    behavior::Behavior,
    message::{Message, SharedStateMessage, SoldierMessage},
    order::Order,
    types::{Angle, SoldierIndex},
};

impl Engine {
    ///  - Compute visibility with other soldiers
    ///  - Compute behavior against physics (explosions, gunfires, ...)
    pub fn animate_soldier(&self, soldier_index: SoldierIndex) -> Vec<Message> {
        let soldier = self.shared_state.soldier(soldier_index);
        if !soldier.can_be_animated() {
            return vec![];
        }

        let mut messages = vec![];

        // Take new order for squad leader
        if self.soldier_is_squad_leader(soldier_index) {
            if let Some(order) = self.shared_state.order_for_squad_leader(soldier_index) {
                if order != soldier.order() {
                    messages.extend(self.take_order(soldier_index, order));
                    messages.push(Message::SharedState(SharedStateMessage::RemoveCommandOder(
                        soldier.squad_uuid(),
                    )));
                }
            }

        // Take new order for squad member
        } else {
            if let Some(order) = self.shared_state.order_for_squad_member(soldier_index) {
                if order != soldier.order() {
                    messages.extend(self.take_order(soldier_index, order));
                    messages.push(Message::SharedState(SharedStateMessage::RemoveSquadOder(
                        soldier_index,
                    )));
                }
            }
        }

        // Adapt behavior according to feelings
        messages.extend(self.adapt_behavior(soldier_index));

        messages
    }

    // FIXME BS NOW : what about all times computed (network, change regulary, etc) ?
    pub fn adapt_behavior(&self, soldier_index: SoldierIndex) -> Vec<Message> {
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
