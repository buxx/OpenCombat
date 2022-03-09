use ggez::mint;
use serde::{Deserialize, Serialize};

use glam::Vec2;

use crate::entity::Entity;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldPoint {
    pub x: f32,
    pub y: f32,
}

impl WorldPoint {
    pub fn apply(self, raw: Vec2) -> Self {
        Self {
            x: self.x + raw.x,
            y: self.y + raw.y,
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl From<Vec2> for WorldPoint {
    fn from(p: Vec2) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl Into<Vec2> for WorldPoint {
    fn into(self) -> Vec2 {
        Vec2::new(self.x.into(), self.y.into())
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct GridPoint {
    pub x: i32,
    pub y: i32,
}

impl GridPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScenePoint {
    pub x: f32,
    pub y: f32,
}

impl ScenePoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn apply(self, raw: Vec2) -> Self {
        Self {
            x: self.x + raw.x,
            y: self.y + raw.y,
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl From<Vec2> for ScenePoint {
    fn from(p: Vec2) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl Into<Vec2> for ScenePoint {
    fn into(self) -> Vec2 {
        Vec2::new(self.x.into(), self.y.into())
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowPoint {
    pub x: f32,
    pub y: f32,
}

impl WindowPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn apply(self, raw: Vec2) -> Self {
        Self {
            x: self.x + raw.x,
            y: self.y + raw.y,
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl From<Vec2> for WindowPoint {
    fn from(p: Vec2) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl Into<Vec2> for WindowPoint {
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
