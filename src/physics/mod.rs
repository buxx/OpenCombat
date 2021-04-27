pub mod position;
pub mod util;

#[derive(Debug)]
pub enum PhysicEvent {
    Explosion,
}

#[derive(Debug)]
pub enum MetaEvent {
    FeelExplosion,
}
