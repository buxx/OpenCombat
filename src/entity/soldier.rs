use crate::{behavior::Behavior, types::*};
use serde::{Deserialize, Serialize};

use super::{Entity, EntityType};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Soldier {
    world_position: WorldPosition,
    squad_uuid: SquadUuid,
    behavior: Behavior,
}

impl Soldier {
    pub fn new(world_position: WorldPosition, squad_uuid: SquadUuid) -> Self {
        Self {
            world_position,
            squad_uuid,
            behavior: Behavior::Idle,
        }
    }

    pub fn from_entity(entity: &ThreadSafeEntity) -> Self {
        Self::new(entity.get_world_position(), entity.squad_uuid())
    }
}

impl Entity for Soldier {
    fn get_type(&self) -> EntityType {
        EntityType::Soldier
    }

    fn get_world_position(&self) -> WorldPosition {
        self.world_position
    }

    fn set_world_position(&mut self, position: WorldPosition) {
        self.world_position = position
    }

    fn squad_uuid(&self) -> SquadUuid {
        self.squad_uuid
    }

    fn get_behavior(&self) -> &Behavior {
        &self.behavior
    }

    fn set_behavior(&mut self, behavior: Behavior) {
        self.behavior = behavior
    }
}
