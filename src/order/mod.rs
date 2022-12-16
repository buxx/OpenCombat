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
            },
            OnBoardPlace::MainCommandment => match self {
                Order::MoveTo(paths) => Behavior::CommandDriveTo(paths.clone()),
                Order::MoveFastTo(paths) => Behavior::CommandDriveTo(paths.clone()),
                Order::SneakTo(paths) => Behavior::CommandDriveTo(paths.clone()),
                Order::Defend(angle) => Behavior::CommandRotateTo(angle.clone()),
                Order::Hide(angle) => Behavior::CommandRotateTo(angle.clone()),
            },
            OnBoardPlace::MainTurretGunner => match self {
                Order::MoveTo(_) => Behavior::Idle,
                Order::MoveFastTo(_) => Behavior::Idle,
                Order::SneakTo(_) => Behavior::Idle,
                Order::Defend(_) => Behavior::Idle,
                Order::Hide(_) => Behavior::Idle,
            },
            OnBoardPlace::Passenger1 => match self {
                Order::MoveTo(_) => Behavior::Idle,
                Order::MoveFastTo(_) => Behavior::Idle,
                Order::SneakTo(_) => Behavior::Idle,
                Order::Defend(_) => Behavior::Idle,
                Order::Hide(_) => Behavior::Idle,
            },
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

    pub fn angle(&self) -> Option<Angle> {
        match self {
            Order::MoveTo(_) | Order::MoveFastTo(_) | Order::SneakTo(_) => None,
            Order::Defend(angle) => Some(*angle),
            Order::Hide(angle) => Some(*angle),
        }
    }
}
