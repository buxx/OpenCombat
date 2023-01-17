use super::Engine;
use crate::{
    behavior::{Behavior, BehaviorMode, BehaviorPropagation},
    entity::soldier::Soldier,
    message::{Message, SharedStateMessage, SoldierMessage},
    order::Order,
    types::{Angle, SquadUuid, WorldPaths},
    utils::DebugPoint,
};

mod blast;
mod bullet;
mod death;
mod defend;
mod moves;

impl Engine {
    pub fn soldier_behavior_mode(&self, soldier: &Soldier) -> BehaviorMode {
        if self.shared_state.soldier_board(soldier.uuid()).is_some() {
            return BehaviorMode::Vehicle;
        }
        BehaviorMode::Ground
    }
    pub fn squad_behavior_mode(&self, squad_index: &SquadUuid) -> BehaviorMode {
        let squad = self.shared_state.squad(*squad_index);
        self.soldier_behavior_mode(self.shared_state.soldier(squad.leader()))
    }

    pub fn soldier_behavior(&self, soldier: &Soldier) -> Vec<Message> {
        let mut messages = vec![];
        let soldier = self.shared_state.soldier(soldier.uuid());

        let behavior = match soldier.order() {
            Order::Idle => self.idle_behavior(soldier),
            Order::MoveTo(paths) => self.move_behavior(soldier, paths),
            Order::MoveFastTo(paths) => self.move_fast_behavior(soldier, paths),
            Order::SneakTo(paths) => self.move_fast_behavior(soldier, paths),
            Order::Defend(angle) => self.defend_behavior(soldier, angle),
            Order::Hide(angle) => self.hide_behavior(soldier, angle),
        };

        // In case of squad leader and regularly propagation
        if self.soldier_is_squad_leader(soldier.uuid())
            && behavior.propagation() == BehaviorPropagation::Regularly
        {
            // Order must be propagated to squad members
            messages.extend(self.propagate_behavior(soldier, &behavior));
        }

        // Change behavior if computed behavior is different
        if &behavior != soldier.behavior() {
            // In case of squad leader and regularly propagation
            if self.soldier_is_squad_leader(soldier.uuid())
                && behavior.propagation() == BehaviorPropagation::OnChange
            {
                // Order must be propagated to squad members
                messages.extend(self.propagate_behavior(soldier, &behavior));
            }

            messages.extend(vec![Message::SharedState(SharedStateMessage::Soldier(
                soldier.uuid(),
                SoldierMessage::SetBehavior(behavior),
            ))]);
        };

        messages
    }

    pub fn propagate_behavior(&self, leader: &Soldier, behavior: &Behavior) -> Vec<Message> {
        assert!(self.soldier_is_squad_leader(leader.uuid()));
        let mut messages = vec![];
        let mut debug_points: Vec<DebugPoint> = vec![];

        let orders: Vec<(&Soldier, Order)> = match behavior {
            Behavior::MoveTo(_) | Behavior::MoveFastTo(_) | Behavior::SneakTo(_) => {
                match self.soldier_behavior_mode(leader) {
                    BehaviorMode::Ground => self.propagate_move(leader.squad_uuid(), &behavior),
                    BehaviorMode::Vehicle => self.propagate_drive(leader.squad_uuid(), &behavior),
                }
            }
            Behavior::Defend(_) => {
                //
                let (orders, debug_points_) = self.propagate_defend(leader.squad_uuid(), &behavior);
                debug_points.extend(debug_points_);
                orders
            }
            Behavior::Hide(_) => {
                // TODO : Special behavior for hide
                let (orders, debug_points_) = self.propagate_defend(leader.squad_uuid(), &behavior);
                debug_points.extend(debug_points_);
                orders
            }
            Behavior::DriveTo(_) => todo!(),
            Behavior::RotateTo(_) => todo!(),
            Behavior::Idle | Behavior::Dead | Behavior::Unconscious => {
                vec![]
            } // Behavior::EngageSoldier(_) => todo!(),
        };

        for (subordinate, order) in orders {
            // Give order only if different from subordinate current order
            if subordinate.order() != &order {
                messages.extend(vec![Message::SharedState(SharedStateMessage::Soldier(
                    subordinate.uuid(),
                    SoldierMessage::SetOrder(order),
                ))]);
            }
        }

        messages
    }

    pub fn idle_behavior(&self, soldier: &Soldier) -> Behavior {
        if let Some(_opponent) = self.get_soldier_opponent(soldier) {
            // return Behavior::EngageSoldier(opponent.uuid());
            return Behavior::Idle;
        }

        if soldier.under_fire().exist() {
            // TODO : soldier angle
            Behavior::Hide(Angle(0.))
        } else {
            Behavior::Idle
        }
    }

    pub fn move_behavior(&self, soldier: &Soldier, paths: &WorldPaths) -> Behavior {
        match self.soldier_behavior_mode(soldier) {
            BehaviorMode::Ground => {
                if soldier.under_fire().is_warning()
                    || soldier.under_fire().is_danger()
                    || soldier.under_fire().is_max()
                {
                    Behavior::SneakTo(paths.clone())
                } else {
                    Behavior::MoveTo(paths.clone())
                }
            }
            BehaviorMode::Vehicle => Behavior::DriveTo(paths.clone()),
        }
    }

    pub fn move_fast_behavior(&self, soldier: &Soldier, paths: &WorldPaths) -> Behavior {
        if soldier.under_fire().is_danger() || soldier.under_fire().is_max() {
            Behavior::SneakTo(paths.clone())
        } else {
            Behavior::MoveFastTo(paths.clone())
        }
    }

    pub fn defend_behavior(&self, _soldier: &Soldier, angle: &Angle) -> Behavior {
        Behavior::Defend(*angle)
    }

    pub fn hide_behavior(&self, _soldier: &Soldier, angle: &Angle) -> Behavior {
        Behavior::Hide(*angle)
    }
}
