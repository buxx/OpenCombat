use crate::physics::visibility::Visibility;

pub mod hit;
pub mod item;
pub mod path;
pub mod projectile;
pub mod util;
pub mod visibility;

#[derive(Debug)]
pub enum PhysicEvent {
    BulletFire(Visibility),
}

#[derive(Debug)]
pub enum MetaEvent {
    FeelExplosion,
}

#[derive(Debug)]
pub enum HitType {
    Deadly,
    Incapacity,
    // TODO: manage these cases
    // Hurting,
    // VeryClose,
    Missed,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct GridPoint {
    pub x: i32,
    pub y: i32,
}

impl GridPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
