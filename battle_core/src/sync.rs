use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    entity::{soldier::Soldier, vehicle::Vehicle},
    types::{SoldiersOnBoard, SquadComposition, SquadUuid},
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct BattleStateCopy {
    soldiers: Vec<Soldier>,
    vehicles: Vec<Vehicle>,
    soldier_on_board: SoldiersOnBoard,
    squads: HashMap<SquadUuid, SquadComposition>,
}

impl BattleStateCopy {
    pub fn new(
        soldiers: Vec<Soldier>,
        vehicles: Vec<Vehicle>,
        soldier_on_board: SoldiersOnBoard,
        squads: HashMap<SquadUuid, SquadComposition>,
    ) -> BattleStateCopy {
        Self {
            soldiers,
            vehicles,
            soldier_on_board,
            squads,
        }
    }

    pub fn soldiers(&self) -> &Vec<Soldier> {
        &self.soldiers
    }

    pub fn vehicles(&self) -> &Vec<Vehicle> {
        &&self.vehicles
    }

    pub fn soldier_on_board(&self) -> &SoldiersOnBoard {
        &self.soldier_on_board
    }

    pub fn squads(&self) -> &HashMap<SquadUuid, SquadComposition> {
        &self.squads
    }
}
