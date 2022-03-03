use crate::types::*;
pub mod soldier;

pub trait Entity {
    fn get_world_position(&self) -> WorldPosition;
    fn set_world_position(&mut self, position: WorldPosition);
    fn squad_uuid(&self) -> SquadUuid;
}
