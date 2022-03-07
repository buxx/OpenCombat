use crate::{behavior::Behavior, types::*};
pub mod soldier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Soldier,
}

pub trait Entity {
    fn get_world_position(&self) -> WorldPosition;
    fn set_world_position(&mut self, position: WorldPosition);
    fn get_behavior(&self) -> &Behavior;
    fn set_behavior(&mut self, behavior: Behavior);
    fn squad_uuid(&self) -> SquadUuid;
    fn get_type(&self) -> EntityType;
}
