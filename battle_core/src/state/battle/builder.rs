use std::{collections::HashMap, fmt::Display};

use crate::{
    deployment::SquadTypes,
    game::flag::FlagsOwnership,
    map::{reader::MapReaderError, Map},
};

use super::{phase::Phase, BattleState};

pub struct BattleStateBuilder {
    map: Map,
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
    pub fn new(map: Map) -> Self {
        Self { map }
    }

    pub fn build(&self) -> Result<BattleState, BattleStateBuilderError> {
        let mut state = BattleState::new(
            0,
            self.map.clone(),
            vec![],
            vec![],
            HashMap::new(),
            SquadTypes::new(),
            Phase::Placement,
            FlagsOwnership::empty(),
        );
        state.resolve();
        Ok(state)
    }
}
