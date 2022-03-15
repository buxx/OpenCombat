use crate::{
    config::{MOVE_FAST_VELOCITY, MOVE_HIDE_VELOCITY, MOVE_VELOCITY},
    types::*,
};
use serde::{Deserialize, Serialize};

pub mod move_;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    Idle,
    MoveTo(WorldPaths),
    MoveFastTo(WorldPaths),
    SneakTo(WorldPaths),
}

impl Behavior {
    pub fn looking_point(&self) -> Option<WorldPoint> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(paths) => paths.next_point(),
            Behavior::MoveFastTo(paths) => paths.next_point(),
            Behavior::SneakTo(paths) => paths.next_point(),
        }
    }

    pub fn velocity(&self) -> Option<f32> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(_) => Some(MOVE_VELOCITY),
            Behavior::MoveFastTo(_) => Some(MOVE_FAST_VELOCITY),
            Behavior::SneakTo(_) => Some(MOVE_HIDE_VELOCITY),
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
        }
    }
}
