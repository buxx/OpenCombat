use std::fmt::Display;

use crate::types::*;
use serde::{Deserialize, Serialize};

use self::marker::OrderMarker;

pub mod marker;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PendingOrder {
    MoveTo(SquadUuid, Option<OrderMarkerIndex>, Vec<WorldPoint>),
    MoveFastTo(SquadUuid, Option<OrderMarkerIndex>, Vec<WorldPoint>),
    SneakTo(SquadUuid, Option<OrderMarkerIndex>, Vec<WorldPoint>),
    // FIXME BS NOW ///////////// delete Angle here no ?
    Defend(SquadUuid, Angle),
    Hide(SquadUuid, Angle),
}

impl PendingOrder {
    pub fn expect_path_finding(&self) -> bool {
        match self {
            PendingOrder::MoveTo(_, _, _)
            | PendingOrder::MoveFastTo(_, _, _)
            | PendingOrder::SneakTo(_, _, _) => true,
            _ => false,
        }
    }

    pub fn marker(&self) -> OrderMarker {
        match self {
            PendingOrder::MoveTo(_, _, _) => OrderMarker::MoveTo,
            PendingOrder::MoveFastTo(_, _, _) => OrderMarker::MoveFastTo,
            PendingOrder::SneakTo(_, _, _) => OrderMarker::SneakTo,
            PendingOrder::Defend(_, _) => OrderMarker::Defend,
            PendingOrder::Hide(_, _) => OrderMarker::Hide,
        }
    }

    pub fn squad_index(&self) -> &SquadUuid {
        match self {
            PendingOrder::MoveTo(squad_index, _, _) => squad_index,
            PendingOrder::MoveFastTo(squad_index, _, _) => squad_index,
            PendingOrder::SneakTo(squad_index, _, _) => squad_index,
            PendingOrder::Defend(squad_index, _) => squad_index,
            PendingOrder::Hide(squad_index, _) => squad_index,
        }
    }

    pub fn order_marker_index(&self) -> &Option<OrderMarkerIndex> {
        match self {
            PendingOrder::MoveTo(_, order_marker_index, _) => order_marker_index,
            PendingOrder::MoveFastTo(_, order_marker_index, _) => order_marker_index,
            PendingOrder::SneakTo(_, order_marker_index, _) => order_marker_index,
            PendingOrder::Defend(_, _) => &None,
            PendingOrder::Hide(_, _) => &None,
        }
    }

    pub fn cached_points(&self) -> Vec<WorldPoint> {
        match self {
            PendingOrder::MoveTo(_, _, cached_points) => cached_points.clone(),
            PendingOrder::MoveFastTo(_, _, cached_points) => cached_points.clone(),
            PendingOrder::SneakTo(_, _, cached_points) => cached_points.clone(),
            PendingOrder::Defend(_, _) => vec![],
            PendingOrder::Hide(_, _) => vec![],
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
