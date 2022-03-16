use serde::{Deserialize, Serialize};

use glam::Vec2;

use crate::entity::Entity;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldPoint {
    pub x: f32,
    pub y: f32,
}

impl WorldPoint {
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

    pub fn _apply(self, raw: Vec2) -> Self {
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

    pub fn next_point(&self) -> Option<WorldPoint> {
        if self.points.is_empty() {
            None
        } else {
            Some(self.points[0])
        }
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn remove_next_point(&mut self) -> Option<WorldPoint> {
        if self.points.is_empty() {
            None
        } else {
            Some(self.points.remove(0))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldPaths {
    pub paths: Vec<WorldPath>,
}

impl WorldPaths {
    pub fn new(paths: Vec<WorldPath>) -> Self {
        Self { paths }
    }

    pub fn next_point(&self) -> Option<WorldPoint> {
        if self.paths.is_empty() {
            None
        } else {
            self.paths[0].next_point()
        }
    }

    pub fn remove_next_point(&mut self) -> Option<WorldPoint> {
        while let Some(path) = self.paths.first_mut() {
            let point = path
                .remove_next_point()
                .expect("We must use WorldPath.remove_next_point() only on feeded paths");
            if path.len() == 0 {
                self.paths.remove(0);
            }
            return Some(point);
        }

        None
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

    pub fn members(&self) -> &Vec<EntityIndex> {
        &self.1
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn _apply(self, raw: Vec2) -> Self {
        Self {
            x: self.x + raw.x,
            y: self.y + raw.y,
        }
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Angle(pub f32);
