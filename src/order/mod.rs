use std::fmt::Display;

use crate::{behavior::Behavior, entity::vehicle::OnBoardPlace, types::*};
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
}

impl Order {
    pub fn to_ground_behavior(&self) -> Behavior {
        match self {
            Order::MoveTo(paths) => Behavior::MoveTo(paths.clone()),
            Order::MoveFastTo(paths) => Behavior::MoveFastTo(paths.clone()),
            Order::SneakTo(paths) => Behavior::SneakTo(paths.clone()),
            Order::Defend(angle) => Behavior::Defend(angle.clone()),
            Order::Hide(angle) => Behavior::Hide(angle.clone()),
            Order::Idle => Behavior::Idle,
        }
    }
    pub fn to_vehicle_behavior(&self, place: &OnBoardPlace) -> Behavior {
        match place {
            OnBoardPlace::Driver => match self {
                Order::MoveTo(paths) => Behavior::DriveTo(paths.clone()),
                Order::MoveFastTo(paths) => Behavior::DriveTo(paths.clone()),
                Order::SneakTo(paths) => Behavior::DriveTo(paths.clone()),
                Order::Defend(angle) => Behavior::RotateTo(angle.clone()),
                Order::Hide(angle) => Behavior::RotateTo(angle.clone()),
                Order::Idle => Behavior::Idle,
            },
            OnBoardPlace::MainCommandment => match self {
                Order::MoveTo(paths) => Behavior::CommandDriveTo(paths.clone()),
                Order::MoveFastTo(paths) => Behavior::CommandDriveTo(paths.clone()),
                Order::SneakTo(paths) => Behavior::CommandDriveTo(paths.clone()),
                Order::Defend(angle) => Behavior::CommandRotateTo(angle.clone()),
                Order::Hide(angle) => Behavior::CommandRotateTo(angle.clone()),
                Order::Idle => Behavior::Idle,
            },
            OnBoardPlace::MainTurretGunner => match self {
                Order::MoveTo(_) => Behavior::Idle,
                Order::MoveFastTo(_) => Behavior::Idle,
                Order::SneakTo(_) => Behavior::Idle,
                Order::Defend(_) => Behavior::Idle,
                Order::Hide(_) => Behavior::Idle,
                Order::Idle => Behavior::Idle,
            },
            OnBoardPlace::Passenger1 => match self {
                Order::MoveTo(_) => Behavior::Idle,
                Order::MoveFastTo(_) => Behavior::Idle,
                Order::SneakTo(_) => Behavior::Idle,
                Order::Defend(_) => Behavior::Idle,
                Order::Hide(_) => Behavior::Idle,
                Order::Idle => Behavior::Idle,
            },
        }
    }

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
