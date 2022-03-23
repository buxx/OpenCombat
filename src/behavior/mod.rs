use crate::{
    config::{MOVE_FAST_VELOCITY, MOVE_HIDE_VELOCITY, MOVE_VELOCITY},
    order::Order,
    types::*,
    utils::angle,
};
use serde::{Deserialize, Serialize};

pub mod movement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    Idle,
    MoveTo(WorldPaths),
    MoveFastTo(WorldPaths),
    SneakTo(WorldPaths),
    Defend(Angle),
    Hide(Angle),
}

impl Behavior {
    pub fn angle(&self, reference_point: WorldPoint) -> Option<Angle> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(paths) => Some(angle(
                &paths.next_point().expect("Must contains point"),
                &reference_point,
            )),
            Behavior::MoveFastTo(paths) => Some(angle(
                &paths.next_point().expect("Must contains point"),
                &reference_point,
            )),
            Behavior::SneakTo(paths) => Some(angle(
                &paths.next_point().expect("Must contains point"),
                &reference_point,
            )),
            Behavior::Defend(angle) => Some(*angle),
            Behavior::Hide(angle) => Some(*angle),
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
        }
    }

    pub fn reach_step(&mut self) {
        match self {
            Behavior::Idle => unreachable!(),
            Behavior::MoveTo(paths) | Behavior::MoveFastTo(paths) | Behavior::SneakTo(paths) => {
                paths
                    .remove_next_point()
                    .expect("Reach a move behavior implies containing point");
            }
            Behavior::Defend(_) => unreachable!(),
            Behavior::Hide(_) => unreachable!(),
        }
    }

    pub fn as_order(&self) -> Option<Order> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(paths) => Some(Order::MoveTo(paths.clone())),
            Behavior::MoveFastTo(paths) => Some(Order::MoveFastTo(paths.clone())),
            Behavior::SneakTo(paths) => Some(Order::SneakTo(paths.clone())),
            Behavior::Defend(angle) => Some(Order::Defend(*angle)),
            Behavior::Hide(angle) => Some(Order::Hide(*angle)),
        }
    }
}
