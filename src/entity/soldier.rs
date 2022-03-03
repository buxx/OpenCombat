use crate::types::*;

use super::Entity;

pub struct Soldier {
    pub world_position: WorldPosition,
}

impl Soldier {
    pub fn new(world_position: WorldPosition) -> Self {
        Self { world_position }
    }
}

impl Entity for Soldier {
    fn world_position(&self) -> WorldPosition {
        self.world_position
    }

    fn set_world_position(&mut self, new_world_position: WorldPosition) {
        self.world_position = new_world_position
    }
}
