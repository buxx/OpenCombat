use crate::types::*;

pub trait Entity {
    fn world_position(&self) -> WorldPosition;
}
