pub mod util;

#[derive(Debug)]
pub enum PhysicEvent {
    Explosion,
}

#[derive(Debug)]
pub enum MetaEvent {
    FeelExplosion,
}

#[derive(Eq, PartialEq, Hash)]
pub struct GridPosition {
    x: i32,
    y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
