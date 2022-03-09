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
