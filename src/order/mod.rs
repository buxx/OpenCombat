use crate::{behavior::Behavior, types::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Order {
    MoveTo(WorldPaths),
}

impl Order {
    pub fn to_behavior(&self) -> Behavior {
        match self {
            Order::MoveTo(paths) => Behavior::MoveTo(paths.clone()),
        }
    }
}
