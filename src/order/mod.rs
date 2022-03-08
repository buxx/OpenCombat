use crate::{behavior::Behavior, types::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Order {
    WalkTo(Vec<WorldPath>),
}

impl Order {
    pub fn to_behavior(&self) -> Behavior {
        match self {
            Order::WalkTo(paths) => Behavior::WalkingTo(paths.clone()),
        }
    }
}
