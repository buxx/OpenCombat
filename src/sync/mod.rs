use serde::{Deserialize, Serialize};

use crate::{entity::soldier::Soldier, state::shared::SharedState};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct StateCopy {
    soldiers: Vec<Soldier>,
}

impl StateCopy {
    pub fn from_state(shared_state: &SharedState) -> StateCopy {
        let mut soldiers = vec![];

        for entity in shared_state.entities() {
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
