use crate::{config::MOVE_VELOCITY, types::*};
use serde::{Deserialize, Serialize};

pub mod walking;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    Idle,
    MoveTo(WorldPaths),
}

impl Behavior {
    pub fn looking_point(&self) -> Option<WorldPoint> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(paths) => paths.next_point(),
        }
    }

    pub fn velocity(&self) -> Option<f32> {
        match self {
            Behavior::Idle => None,
            Behavior::MoveTo(_) => Some(MOVE_VELOCITY),
        }
    }

    pub fn reach_step(&mut self) {
        match self {
            Behavior::Idle => unreachable!(),
            Behavior::MoveTo(paths) => {
                paths
                    .remove_next_point()
                    .expect("Reach a move behavior implies containing point");
            }
        }
    }
}
