use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::{
    game::flag::FlagsOwnership,
    map::reader::{MapReader, MapReaderError},
};

use super::{phase::Phase, BattleState};

pub struct BattleStateBuilder {
    map_name: String,
    resources: PathBuf,
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
    pub fn new(map_name: &str, resources: PathBuf) -> Self {
        Self {
            map_name: map_name.to_string(),
            resources,
        }
    }

    pub fn build(&self) -> Result<BattleState, BattleStateBuilderError> {
        let map = MapReader::new(&self.map_name, &self.resources)?.build()?;
        let mut state = BattleState::new(
            map,
            vec![],
            vec![],
            HashMap::new(),
            Phase::Placement,
            FlagsOwnership::empty(),
        );
        state.resolve();
        Ok(state)
    }
}
