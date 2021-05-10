pub mod util;

#[derive(Debug)]
pub enum PhysicEvent {
    Explosion,
}

#[derive(Debug)]
pub enum MetaEvent {
    FeelExplosion,
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct GridPoint {
    x: i32,
    y: i32,
}

impl GridPoint {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
