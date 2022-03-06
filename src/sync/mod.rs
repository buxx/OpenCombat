use serde::{Deserialize, Serialize};

use crate::{entity::soldier::Soldier, state::State};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct StateCopy {
    soldiers: Vec<Soldier>,
}

impl StateCopy {
    pub fn from_state(state: &State) -> StateCopy {
        let mut soldiers = vec![];

        for entity in state.entities() {
            // FIXME BS NOW ....
            soldiers.push(Soldier::new(
                entity.get_world_position(),
                entity.squad_uuid(),
            ))
        }

        Self { soldiers }
    }

    pub fn soldiers(&self) -> &Vec<Soldier> {
        &self.soldiers
    }
}
