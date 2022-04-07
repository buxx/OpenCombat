use crate::{
    config::{MOVE_FAST_VELOCITY, MOVE_HIDE_VELOCITY, MOVE_VELOCITY},
    order::Order,
    types::*,
    utils::angle,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    // Ground specific orders
    MoveTo(WorldPaths),
    MoveFastTo(WorldPaths),
    SneakTo(WorldPaths),
    // Vehicle specific orders
    CommandDriveTo(WorldPaths),
    CommandRotateTo(Angle),
    DriveTo(WorldPaths),
    RotateTo(Angle),
    // Common orders
    Idle,
    Defend(Angle),
    Hide(Angle),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorMode {
    Ground,
    Vehicle,
}

impl Behavior {
    pub fn angle(&self, reference_point: WorldPoint) -> Option<Angle> {
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
            Behavior::CommandDriveTo(_) => None,
            Behavior::CommandRotateTo(_) => None,
            Behavior::DriveTo(_) => None,
            Behavior::RotateTo(_) => None,
        }
    }

    pub fn velocity(&self) -> Option<f32> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(_) => Some(MOVE_VELOCITY),
            Behavior::MoveFastTo(_) => Some(MOVE_FAST_VELOCITY),
            Behavior::SneakTo(_) => Some(MOVE_HIDE_VELOCITY),
            Behavior::CommandDriveTo(_) => None,
            Behavior::Defend(_) => None,
            Behavior::Hide(_) => None,
            Behavior::CommandRotateTo(_) => None,
            Behavior::DriveTo(_) => None,
            Behavior::RotateTo(_) => None,
        }
    }

    pub fn reach_step(&mut self) -> bool {
        match self {
            // FIXME BS NOW : Look like client reach it when client started before server
            Behavior::MoveTo(paths)
            | Behavior::MoveFastTo(paths)
            | Behavior::SneakTo(paths)
            | Behavior::DriveTo(paths)
            | Behavior::CommandDriveTo(paths) => {
                paths
                    .remove_next_point()
                    .expect("Reach a move behavior implies containing point");

                if paths.next_point().is_none() {
                    return true;
                }
            }
            Behavior::Idle
            | Behavior::Defend(_)
            | Behavior::Hide(_)
            | Behavior::CommandRotateTo(_)
            | Behavior::RotateTo(_) => {}
        }

        false
    }

    pub fn to_order(&self) -> Option<Order> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(paths) => Some(Order::MoveTo(paths.clone())),
            Behavior::MoveFastTo(paths) => Some(Order::MoveFastTo(paths.clone())),
            Behavior::SneakTo(paths) => Some(Order::SneakTo(paths.clone())),
            Behavior::Defend(angle) => Some(Order::Defend(*angle)),
            Behavior::Hide(angle) => Some(Order::Hide(*angle)),
            Behavior::CommandDriveTo(paths) => Some(Order::MoveTo(paths.clone())),
            Behavior::CommandRotateTo(angle) => None,
            Behavior::DriveTo(paths) => Some(Order::MoveTo(paths.clone())),
            Behavior::RotateTo(angle) => None,
        }
    }
}
