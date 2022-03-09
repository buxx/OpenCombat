use crate::types::*;
use serde::{Deserialize, Serialize};

pub mod walking;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    Idle,
    WalkingTo(Vec<WorldPath>),
}

impl Behavior {
    pub fn looking_point(&self) -> Option<WorldPoint> {
        match self {
            Behavior::Idle => None,
            Behavior::WalkingTo(paths) => {
                if paths.is_empty() {
                    None
                } else {
                    // Note : WalkingTo Behavior must exist only if containing at least one path with one point
                    Some(paths[0].points[0])
                }
            }
        }
    }
}
