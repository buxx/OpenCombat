use std::fmt::Display;

use crate::types::*;
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
            Behavior::SuppressFire(_) => BehaviorPropagation::OnChange,
            Behavior::Dead => BehaviorPropagation::Never,
            Behavior::Unconscious => BehaviorPropagation::Never,
            Behavior::EngageSoldier(_) => BehaviorPropagation::OnChange,
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
            | Behavior::Idle
            | Behavior::Defend(_)
            | Behavior::Hide(_)
            | Behavior::Dead
            | Behavior::Unconscious
            | Behavior::SuppressFire(_)
            | Behavior::EngageSoldier(_) => None,
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
            Behavior::Idle => f.write_str("Idle"),
            Behavior::Defend(_) => f.write_str("Defend"),
            Behavior::Hide(_) => f.write_str("Hide"),
            Behavior::Dead => f.write_str("Dead"),
            Behavior::Unconscious => f.write_str("Unconscious"),
            Behavior::SuppressFire(_) => f.write_str("SuppressFire"),
            Behavior::EngageSoldier(_) => f.write_str("EngageSquad"),
        }
    }
}
