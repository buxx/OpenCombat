use std::fmt::Display;

use crate::types::*;
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
    Idle,
    MoveTo(WorldPaths),
    MoveFastTo(WorldPaths),
    SneakTo(WorldPaths),
    Defend(Angle),
    Hide(Angle),
    // EngageSquad(SquadUuid),
}

impl Order {
    pub fn marker(&self) -> Option<OrderMarker> {
        match self {
            Order::MoveTo(_) => Some(OrderMarker::MoveTo),
            Order::MoveFastTo(_) => Some(OrderMarker::MoveFastTo),
            Order::SneakTo(_) => Some(OrderMarker::SneakTo),
            Order::Defend(_) => Some(OrderMarker::Defend),
            Order::Hide(_) => Some(OrderMarker::Hide),
            Order::Idle => None,
        }
    }

    pub fn angle(&self) -> Option<Angle> {
        match self {
            Order::MoveTo(_) | Order::MoveFastTo(_) | Order::SneakTo(_) => None,
            Order::Defend(angle) => Some(*angle),
            Order::Hide(angle) => Some(*angle),
            Order::Idle => None,
        }
    }

    pub fn reach_step(&mut self) -> bool {
        match self {
            Order::MoveTo(paths) | Order::MoveFastTo(paths) | Order::SneakTo(paths) => {
                paths
                    .remove_next_point()
                    .expect("Reach a move behavior implies containing point");

                if paths.next_point().is_none() {
                    return true;
                }
            }
            Order::Defend(_) => {}
            Order::Hide(_) => {}
            Order::Idle => {}
        }

        false
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::MoveTo(_) => f.write_str("MoveTo"),
            Order::MoveFastTo(_) => f.write_str("MoveFastTo"),
            Order::SneakTo(_) => f.write_str("SneakTo"),
            Order::Defend(_) => f.write_str("Defend"),
            Order::Hide(_) => f.write_str("Hide"),
            Order::Idle => f.write_str("Idle"),
        }
    }
}
