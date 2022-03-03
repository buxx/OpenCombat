use crate::types::*;
pub mod soldier;

pub trait Entity {
    fn world_position(&self) -> WorldPosition;
    fn set_world_position(&mut self, new_world_position: WorldPosition);
}
