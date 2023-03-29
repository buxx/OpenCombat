use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::{
    entity::{soldier::Soldier, vehicle::Vehicle},
    map::{
        reader::{MapReader, MapReaderError},
        Map,
    },
    types::SoldiersOnBoard,
};

use crate::hardcode;

use super::{phase::Phase, BattleState};

pub struct BattleStateBuilder {
    map: Map,
    soldiers: Vec<Soldier>,
    vehicles: Vec<Vehicle>,
    soldier_on_board: SoldiersOnBoard,
}

#[derive(Debug)]
pub enum BattleStateBuilderError {
    MapReaderError(MapReaderError),
}

impl From<MapReaderError> for BattleStateBuilderError {
    fn from(error: MapReaderError) -> Self {
        Self::MapReaderError(error)
    }
}

impl Display for BattleStateBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BattleStateBuilderError::MapReaderError(error) => {
                f.write_str(&format!("Map reader error : {}", error))
            }
        }
    }
}

impl BattleStateBuilder {
    pub fn new(map_name: &str, resources: &PathBuf) -> Result<Self, BattleStateBuilderError> {
        Ok(Self {
            map: MapReader::new(map_name, &resources)?.build()?,
            soldiers: vec![],
            vehicles: vec![],
            soldier_on_board: HashMap::new(),
        })
    }

    pub fn situation(mut self, _name: &str) -> Self {
        // TODO : hardcoded for nom, later, will a description file
        let (soldiers, vehicles, soldier_on_board) = hardcode::situation();
        self.soldiers = soldiers;
        self.vehicles = vehicles;
        self.soldier_on_board = soldier_on_board;

        self
    }

    pub fn build(&self) -> BattleState {
        let mut state = BattleState::new(
            self.map.clone(),
            self.soldiers.clone(),
            self.vehicles.clone(),
            self.soldier_on_board.clone(),
            Phase::Placement,
        );
        state.resolve();
        state
    }
}
