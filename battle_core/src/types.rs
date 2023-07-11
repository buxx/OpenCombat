use std::{
    collections::HashMap,
    f32::consts::FRAC_PI_2,
    fmt::Display,
    ops::{Add, Neg},
};

use serde::{Deserialize, Serialize};

use glam::Vec2;

use crate::entity::vehicle::OnBoardPlace;

pub trait Xy {
    fn from_xy(x: f32, y: f32) -> Self;
    fn apply(&self, x: f32, y: f32) -> Self;
    fn x(&self) -> f32;
    fn y(&self) -> f32;
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Xy for Point {
    fn from_xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }

    fn apply(&self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorldPoint {
    pub x: f32,
    pub y: f32,
}

impl WorldPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_vec2(vec: Vec2) -> Self {
        Self { x: vec.x, y: vec.y }
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

impl Xy for WorldPoint {
    fn from_xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }

    fn apply(&self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
        }
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

impl Display for GridPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{},{}", self.x, self.y))
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

impl Xy for WindowPoint {
    fn from_xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }

    fn apply(&self, x: f32, y: f32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2> for Velocity {
    fn from(p: Vec2) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl Into<Vec2> for Velocity {
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

    pub fn last_point(&self) -> Option<WorldPoint> {
        if self.points.is_empty() {
            None
        } else {
            Some(self.points[self.points.len() - 1])
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GridPath {
    pub points: Vec<GridPoint>,
}

impl GridPath {
    pub fn new() -> Self {
        Self { points: vec![] }
    }

    pub fn contains(&self, point: &GridPoint) -> bool {
        self.points.contains(point)
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn push(&mut self, point: GridPoint) {
        self.points.push(point)
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

    pub fn _next_path_last_point(&self) -> Option<WorldPoint> {
        if self.paths.is_empty() {
            None
        } else {
            self.paths[0].last_point()
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

    pub fn is_last_point(&self) -> Option<bool> {
        if self.paths.is_empty() {
            None
        } else if self.paths.len() > 1 {
            Some(false)
        } else {
            Some(self.paths[0].points.len() == 1)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SoldierIndex(pub usize);

impl Display for SoldierIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VehicleIndex(pub usize);

impl Display for VehicleIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VehicleSize(pub usize);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OrderMarkerIndex(pub usize);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct BulletFireIndex(pub usize);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ExplosionIndex(pub usize);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SquadUuid(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SquadComposition(SoldierIndex, Vec<SoldierIndex>);

impl SquadComposition {
    pub fn new(leader: SoldierIndex, members: Vec<SoldierIndex>) -> Self {
        Self(leader, members)
    }

    pub fn leader(&self) -> SoldierIndex {
        self.0
    }

    pub fn members(&self) -> &Vec<SoldierIndex> {
        &self.1
    }

    pub fn subordinates(&self) -> Vec<&SoldierIndex> {
        self.1.iter().filter(|i| i != &&self.0).collect()
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
}

impl Scale {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn apply(&mut self, raw: Vec2) {
        self.x += raw.x;
        self.y += raw.y;
    }

    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl From<Vec2> for Scale {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
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

    pub fn half() -> Self {
        Self { x: 0.5, y: 0.5 }
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

    pub fn from_vec2(vec: Vec2) -> Self {
        Self::new(vec.x, vec.y)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbsoluteOffset {
    pub x: f32,
    pub y: f32,
}

impl AbsoluteOffset {
    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelativeOffset {
    pub x: f32,
    pub y: f32,
}

impl RelativeOffset {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Angle(pub f32);

impl Angle {
    pub fn from_points(to_point: &Vec2, from_point: &Vec2) -> Self {
        Self(f32::atan2(to_point.y - from_point.y, to_point.x - from_point.x) + FRAC_PI_2)
    }

    pub fn zero() -> Self {
        Self(0.)
    }
}

impl Add for Angle {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Distance {
    pub millimeters: i64,
}

impl Distance {
    pub fn from_meters(meters: i64) -> Self {
        Self {
            millimeters: meters * 1000,
        }
    }

    pub fn from_millimeters(millimeters: i64) -> Self {
        Self { millimeters }
    }

    pub fn millimeters(&self) -> i64 {
        self.millimeters
    }

    pub fn meters(&self) -> i64 {
        self.millimeters / 1000
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Precision(u8);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage(pub f32);

pub type SoldierBoard = (VehicleIndex, OnBoardPlace);
pub type SoldiersOnBoard = HashMap<SoldierIndex, SoldierBoard>;
pub type VehicleBoard = HashMap<VehicleIndex, Vec<(OnBoardPlace, SoldierIndex)>>;
pub type BoardComposition = Vec<OnBoardPlace>;
pub type VehicleGraphicPlaces = HashMap<OnBoardPlace, Offset>;
