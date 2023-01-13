use std::fmt::Display;

use crate::{
    config::{MOVE_FAST_VELOCITY, MOVE_HIDE_VELOCITY, MOVE_VELOCITY},
    order::Order,
    state::shared::SharedState,
    types::*,
    utils::angle,
};
use serde::{Deserialize, Serialize};

pub mod feeling;
pub mod gesture;

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
    Idle,
    Defend(Angle),
    Hide(Angle),
    //
    Dead,
    Unconscious,
    // Combat
    // EngageSoldier(SoldierIndex),
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
    pub fn angle(&self, reference_point: WorldPoint, shared_state: &SharedState) -> Option<Angle> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => {
                if let Some(next_point) = paths.next_point() {
                    Some(angle(&next_point, &reference_point))
                } else {
                    None
                }
            }
            Behavior::Defend(angle) => Some(*angle),
            Behavior::Hide(angle) => Some(*angle),
            Behavior::DriveTo(_) => None,
            Behavior::RotateTo(_) => None,
            // Behavior::EngageSoldier(opponent_index) => {
            //     let opponent = shared_state.soldier(*opponent_index);
            //     Some(angle(&opponent.get_world_point(), &reference_point))
            // }
            // TODO: keep angle for dead/unconscious soldiers
            Behavior::Dead | Behavior::Unconscious => None,
        }
    }

    pub fn velocity(&self) -> Option<f32> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(_) => Some(MOVE_VELOCITY),
            Behavior::MoveFastTo(_) => Some(MOVE_FAST_VELOCITY),
            Behavior::SneakTo(_) => Some(MOVE_HIDE_VELOCITY),
            Behavior::Defend(_) => None,
            Behavior::Hide(_) => None,
            Behavior::DriveTo(_) => None,
            Behavior::RotateTo(_) => None,
            Behavior::Dead => None,
            Behavior::Unconscious => None,
            // Behavior::EngageSoldier(_) => None,
        }
    }

    pub fn propagation(&self) -> BehaviorPropagation {
        match self {
            Behavior::MoveTo(_) | Behavior::MoveFastTo(_) | Behavior::SneakTo(_) => {
                BehaviorPropagation::Regularly
            }
            Behavior::DriveTo(_) => BehaviorPropagation::Never,
            Behavior::RotateTo(_) => todo!(),
            Behavior::Idle => BehaviorPropagation::OnChange,
            Behavior::Defend(_) => BehaviorPropagation::OnChange,
            Behavior::Hide(_) => BehaviorPropagation::OnChange,
            Behavior::Dead => BehaviorPropagation::Never,
            Behavior::Unconscious => BehaviorPropagation::Never,
        }
    }

    pub fn reach_step(&mut self) -> bool {
        match self {
            // FIXME BS NOW : Look like client reach it when client started before server
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
            Behavior::Idle | Behavior::Defend(_) | Behavior::Hide(_) | Behavior::RotateTo(_) => {}
            Behavior::Dead => {}
            Behavior::Unconscious => {} // Behavior::EngageSoldier(_) => {}
        }

        false
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
            Behavior::Idle => f.write_str("Idle"),
            Behavior::Defend(_) => f.write_str("Defend"),
            Behavior::Hide(_) => f.write_str("Hide"),
            Behavior::Dead => f.write_str("Dead"),
            Behavior::Unconscious => f.write_str("Unconscious"),
            // Behavior::EngageSoldier(_) => f.write_str("Engage"),
        }
    }
}
