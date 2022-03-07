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
            match entity.get_type() {
                crate::entity::EntityType::Soldier => {
                    soldiers.push(Soldier::from_entity(entity));
                }
            }
        }

        Self { soldiers }
    }

    pub fn soldiers(&self) -> &Vec<Soldier> {
        &self.soldiers
    }
}
