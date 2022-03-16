use crate::{behavior::Behavior, types::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PendingOrder {
    MoveTo,
    MoveFastTo,
    SneakTo,
    Defend,
    Hide,
}

impl PendingOrder {
    pub fn expect_path_finding(&self) -> bool {
        match self {
            PendingOrder::MoveTo | PendingOrder::MoveFastTo | PendingOrder::SneakTo => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Order {
    MoveTo(WorldPaths),
    MoveFastTo(WorldPaths),
    SneakTo(WorldPaths),
    Defend(Angle),
    Hide(Angle),
}

impl Order {
    pub fn to_behavior(&self) -> Behavior {
        match self {
            Order::MoveTo(paths) => Behavior::MoveTo(paths.clone()),
            Order::MoveFastTo(paths) => Behavior::MoveFastTo(paths.clone()),
            Order::SneakTo(paths) => Behavior::SneakTo(paths.clone()),
            Order::Defend(angle) => Behavior::Defend(angle.clone()),
            Order::Hide(angle) => Behavior::Hide(angle.clone()),
        }
    }
}
