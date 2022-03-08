use crate::types::*;
use serde::{Deserialize, Serialize};

pub mod walking;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    Idle,
    WalkingTo(Vec<WorldPath>),
}
