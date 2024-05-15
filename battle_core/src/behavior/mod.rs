use std::fmt::Display;

use crate::{
    config::{CAN_CROUCH_AFTER, CAN_STANDUP_AFTER},
    entity::soldier::Soldier,
    game::posture::Posture,
    order::Order,
    state::battle::BattleState,
    types::*,
};
use serde::{Deserialize, Serialize};

pub mod feeling;
pub mod gesture;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Body {
    StandUp,
    Crouched,
    Lying,
}
impl Body {
    pub fn from_soldier(soldier: &Soldier, battle: &BattleState) -> Self {
        if *soldier.last_shoot_frame_i() == 0 {
            // 0 == never
            return Self::StandUp;
        }

        if soldier.last_shot_frame_i() + CAN_CROUCH_AFTER <= *battle.frame_i() {
            return Self::Crouched;
        }

        if soldier.last_shot_frame_i() + CAN_STANDUP_AFTER <= *battle.frame_i() {
            return Self::StandUp;
        }

        Self::Lying
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    // Ground specific orders
    MoveTo(WorldPaths),
    MoveFastTo(WorldPaths),
    SneakTo(WorldPaths),
    // Vehicle specific orders
    DriveTo(WorldPaths),
    RotateTo(Angle),
    // Common orders
    Idle(Body),
    Defend(Angle),
    Hide(Angle),
    //
    Dead,
    Unconscious,
    // Combat
    SuppressFire(WorldPoint),
    EngageSoldier(SoldierIndex),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorMode {
    Ground,
    Vehicle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorPropagation {
    OnChange,
    Regularly,
    Never,
}

impl Behavior {
    pub fn from_order(order: &Order, soldier: &Soldier, battle_state: &BattleState) -> Self {
        match order {
            Order::Idle => Behavior::Idle(Body::from_soldier(soldier, battle_state)),
            Order::MoveTo(path, _) => Behavior::MoveTo(path.clone()),
            Order::MoveFastTo(path, _) => Behavior::MoveFastTo(path.clone()),
            Order::SneakTo(path, _) => Behavior::SneakTo(path.clone()),
            Order::Defend(angle) => Behavior::Defend(*angle),
            Order::Hide(angle) => Behavior::Hide(*angle),
            // default_behavior should never be called for EngageSquad & SuppressFire
            Order::EngageSquad(_squad_id) => unreachable!(),
            Order::SuppressFire(_point) => unreachable!(),
        }
    }

    pub fn propagation(&self) -> BehaviorPropagation {
        match self {
            Behavior::MoveTo(_) | Behavior::MoveFastTo(_) | Behavior::SneakTo(_) => {
                BehaviorPropagation::Regularly
            }
            Behavior::DriveTo(_) => BehaviorPropagation::Never,
            Behavior::RotateTo(_) => BehaviorPropagation::Never,
            Behavior::Idle(_) => BehaviorPropagation::OnChange,
            Behavior::Defend(_) => BehaviorPropagation::OnChange,
            Behavior::Hide(_) => BehaviorPropagation::OnChange,
            Behavior::SuppressFire(_) => BehaviorPropagation::OnChange,
            Behavior::Dead => BehaviorPropagation::Never,
            Behavior::Unconscious => BehaviorPropagation::Never,
            Behavior::EngageSoldier(_) => BehaviorPropagation::OnChange,
        }
    }

    pub fn reach_step(&mut self) -> bool {
        match self {
            Behavior::MoveTo(paths)
            | Behavior::MoveFastTo(paths)
            | Behavior::SneakTo(paths)
            | Behavior::DriveTo(paths) => {
                paths
                    .remove_next_point()
                    .expect("Reach a move behavior implies containing point");

                if paths.next_point().is_none() {
                    return true;
                }
            }
            Behavior::Idle(_) | Behavior::Defend(_) | Behavior::Hide(_) | Behavior::RotateTo(_) => {
            }
            Behavior::Dead => {}
            Behavior::Unconscious => {}
            Behavior::SuppressFire(_) => {}
            Behavior::EngageSoldier(_) => {}
        }

        false
    }

    pub fn world_paths(&self) -> Option<&WorldPaths> {
        match self {
            Behavior::MoveTo(world_paths)
            | Behavior::MoveFastTo(world_paths)
            | Behavior::SneakTo(world_paths)
            | Behavior::DriveTo(world_paths) => Some(world_paths),
            Behavior::RotateTo(_)
            | Behavior::Idle(_)
            | Behavior::Defend(_)
            | Behavior::Hide(_)
            | Behavior::Dead
            | Behavior::Unconscious
            | Behavior::SuppressFire(_)
            | Behavior::EngageSoldier(_) => None,
        }
    }

    pub fn posture(&self) -> Posture {
        // TODO : posture can be different on same behavior (like with SuppressFire, EngageSoldier)
        // FIXME: Clarify which usage with `Body` !!
        match self {
            Behavior::MoveTo(_) | Behavior::MoveFastTo(_) | Behavior::Idle(_) => Posture::StandUp,
            Behavior::Defend(_)
            | Behavior::SneakTo(_)
            | Behavior::DriveTo(_)
            | Behavior::RotateTo(_)
            | Behavior::Hide(_)
            | Behavior::Dead
            | Behavior::Unconscious
            | Behavior::SuppressFire(_)
            | Behavior::EngageSoldier(_) => Posture::Flat,
        }
    }

    pub fn is_hide(&self) -> bool {
        matches!(self, Behavior::Hide(_))
    }

    pub fn opponent(&self) -> Option<&SoldierIndex> {
        match self {
            Behavior::EngageSoldier(soldier_index) => Some(soldier_index),
            _ => None,
        }
    }
}
impl Display for Body {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Body::StandUp => f.write_str("StandUp"),
            Body::Crouched => f.write_str("Crouched"),
            Body::Lying => f.write_str("Lying"),
        }
    }
}

impl Display for Behavior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Behavior::MoveTo(_) => f.write_str("MoveTo"),
            Behavior::MoveFastTo(_) => f.write_str("MoveFastTo"),
            Behavior::SneakTo(_) => f.write_str("SneakTo"),
            Behavior::DriveTo(_) => f.write_str("DriveTo"),
            Behavior::RotateTo(_) => f.write_str("RotateTo"),
            Behavior::Idle(position) => f.write_str(&format!("Idle {}", position)),
            Behavior::Defend(_) => f.write_str("Defend"),
            Behavior::Hide(_) => f.write_str("Hide"),
            Behavior::Dead => f.write_str("Dead"),
            Behavior::Unconscious => f.write_str("Unconscious"),
            Behavior::SuppressFire(_) => f.write_str("SuppressFire"),
            Behavior::EngageSoldier(_) => f.write_str("EngageSquad"),
        }
    }
}
