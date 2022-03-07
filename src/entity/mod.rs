use crate::types::*;
pub mod soldier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Soldier,
}

pub trait Entity {
    fn get_world_position(&self) -> WorldPosition;
    fn set_world_position(&mut self, position: WorldPosition);
    fn squad_uuid(&self) -> SquadUuid;
    fn type_(&self) -> EntityType;
}
