use serde::{Deserialize, Serialize};

use crate::{
    deployment::SquadTypes,
    entity::{soldier::Soldier, vehicle::Vehicle},
    game::flag::FlagsOwnership,
    state::battle::phase::Phase,
    types::SoldiersOnBoard,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct BattleStateCopy {
    frame_i: u64,
    soldiers: Vec<Soldier>,
    vehicles: Vec<Vehicle>,
    soldier_on_board: SoldiersOnBoard,
    squad_types: SquadTypes,
    phase: Phase,
    flags: FlagsOwnership,
}

impl BattleStateCopy {
    pub fn new(
        frame_i: u64,
        soldiers: Vec<Soldier>,
        vehicles: Vec<Vehicle>,
        soldier_on_board: SoldiersOnBoard,
        squad_types: SquadTypes,
        phase: Phase,
        flags: FlagsOwnership,
    ) -> BattleStateCopy {
        Self {
            frame_i,
            soldiers,
            vehicles,
            soldier_on_board,
            squad_types,
            phase,
            flags,
        }
    }

    pub fn frame_i(&self) -> u64 {
        self.frame_i
    }

    pub fn soldiers(&self) -> &Vec<Soldier> {
        &self.soldiers
    }

    pub fn vehicles(&self) -> &Vec<Vehicle> {
        &self.vehicles
    }

    pub fn soldier_on_board(&self) -> &SoldiersOnBoard {
        &self.soldier_on_board
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn flags(&self) -> &FlagsOwnership {
        &self.flags
    }

    pub fn squad_types(&self) -> &SquadTypes {
        &self.squad_types
    }
}
