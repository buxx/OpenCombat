use crate::{behavior::Behavior, types::*};
use serde::{Deserialize, Serialize};

use self::marker::OrderMarker;

pub mod marker;

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

    pub fn marker(&self) -> OrderMarker {
        match self {
            PendingOrder::MoveTo => OrderMarker::MoveTo,
            PendingOrder::MoveFastTo => OrderMarker::MoveFastTo,
            PendingOrder::SneakTo => OrderMarker::SneakTo,
            PendingOrder::Defend => OrderMarker::Defend,
            PendingOrder::Hide => OrderMarker::Hide,
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

    pub fn marker(&self) -> OrderMarker {
        match self {
            Order::MoveTo(_) => OrderMarker::MoveTo,
            Order::MoveFastTo(_) => OrderMarker::MoveFastTo,
            Order::SneakTo(_) => OrderMarker::SneakTo,
            Order::Defend(_) => OrderMarker::Defend,
            Order::Hide(_) => OrderMarker::Hide,
        }
    }
}
