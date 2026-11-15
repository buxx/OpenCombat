use derive_more::Deref;
use enum_type_derive::EnumType;
use glam::Vec2;
use oc_root::geo::WorldVec2;
use oc_utils::d2::Direction;

use crate::IndividualIndex;

/// Index of squad order in squad order, by starting end
#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, Deref, PartialEq)]
pub struct OrderIndex(pub u32);

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, EnumType)]
pub enum Order {
    Idle,
    MoveTo(WorldVec2),
    MoveFastTo(WorldVec2),
    SneakTo(WorldVec2),
    Defend(Direction),
    Hide(Direction),
    Engage(IndividualIndex),
    Suppress(WorldVec2),
    Grenade(WorldVec2),
    Smoke(WorldVec2),
}

impl Order {
    /// Direction the order implies, seen from `from`. `individual_position` gives the position of
    /// an individual (for engage). None if order implies no direction (idle).
    pub fn direction(
        &self,
        from: Vec2,
        individual_position: impl Fn(IndividualIndex) -> Vec2,
    ) -> Option<Direction> {
        match self {
            Order::Idle => None,
            Order::Defend(direction) | Order::Hide(direction) => Some(*direction),
            Order::MoveTo(target)
            | Order::MoveFastTo(target)
            | Order::SneakTo(target)
            | Order::Suppress(target)
            | Order::Grenade(target)
            | Order::Smoke(target) => Some(Direction::from_points2d(from, (*target).into())),
            Order::Engage(target) => {
                Some(Direction::from_points2d(from, individual_position(*target)))
            }
        }
    }

    /// Return position if order own it (moves / suppress)
    pub fn position(&self) -> Option<WorldVec2> {
        match self {
            Order::Idle | Order::Defend(_) | Order::Hide(_) | Order::Engage(_) => None,
            Order::MoveTo(position)
            | Order::MoveFastTo(position)
            | Order::SneakTo(position)
            | Order::Suppress(position)
            | Order::Grenade(position)
            | Order::Smoke(position) => Some(*position),
        }
    }

    /// Update position if order own one (moves / fires)
    pub fn set_position(&mut self, position: WorldVec2) {
        match self {
            Order::Idle | Order::Defend(_) | Order::Hide(_) | Order::Engage(_) => {}
            Order::MoveTo(position_)
            | Order::MoveFastTo(position_)
            | Order::SneakTo(position_)
            | Order::Suppress(position_)
            | Order::Grenade(position_)
            | Order::Smoke(position_) => *position_ = position,
        }
    }

    pub fn hud_label(&self) -> &str {
        match self {
            Order::Idle => "Idle",
            Order::MoveTo(_) => "Move",
            Order::MoveFastTo(_) => "Move fast",
            Order::SneakTo(_) => "Sneak",
            Order::Defend(_) => "Defend",
            Order::Hide(_) => "Hide",
            Order::Engage(_) => "Engage",
            Order::Suppress(_) => "Suppress",
            Order::Grenade(_) => "Grenade",
            Order::Smoke(_) => "Smoke",
        }
    }

    /// Modify self with position if self is an order which accept it
    pub fn with_position(mut self, position: WorldVec2) -> Order {
        match &mut self {
            Order::MoveTo(p)
            | Order::MoveFastTo(p)
            | Order::SneakTo(p)
            | Order::Suppress(p)
            | Order::Grenade(p)
            | Order::Smoke(p) => *p = position,
            Order::Idle | Order::Defend(_) | Order::Hide(_) | Order::Engage(_) => {}
        }

        self
    }
}
