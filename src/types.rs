use serde::{Deserialize, Serialize};
use std::ops::Add;

use glam::Vec2;

use crate::entity::Entity;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldX(f32);

impl From<f32> for WorldX {
    fn from(x: f32) -> Self {
        Self(x)
    }
}

impl Into<f32> for WorldX {
    fn into(self) -> f32 {
        self.0
    }
}

impl Add for WorldX {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldY(f32);

impl From<f32> for WorldY {
    fn from(y: f32) -> Self {
        Self(y)
    }
}

impl Into<f32> for WorldY {
    fn into(self) -> f32 {
        self.0
    }
}

impl Add for WorldY {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldPoint {
    pub x: WorldX,
    pub y: WorldY,
}

impl WorldPoint {
    pub fn apply_raw(self, raw: Vec2) -> Self {
        Self {
            x: WorldX(self.x.0 + raw.x),
            y: WorldY(self.y.0 + raw.y),
        }
    }
}

impl From<(WorldX, WorldY)> for WorldPoint {
    fn from(p: (WorldX, WorldY)) -> Self {
        Self { x: p.0, y: p.1 }
    }
}

impl Into<Vec2> for WorldPoint {
    fn into(self) -> Vec2 {
        Vec2::new(self.x.into(), self.y.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldPath {
    pub points: Vec<WorldPoint>,
}

impl WorldPath {
    pub fn new(points: Vec<WorldPoint>) -> Self {
        Self { points }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EntityIndex(pub usize);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SquadUuid(pub usize);
pub type ThreadSafeEntity = Box<dyn Entity + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SquadComposition(EntityIndex, Vec<EntityIndex>);

impl SquadComposition {
    pub fn new(leader: EntityIndex, members: Vec<EntityIndex>) -> Self {
        Self(leader, members)
    }

    pub fn leader(&self) -> EntityIndex {
        self.0
    }

    pub fn _members(&self) -> &Vec<EntityIndex> {
        &self.1
    }
}
