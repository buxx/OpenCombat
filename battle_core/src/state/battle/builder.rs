use std::{collections::HashMap, fmt::Display, path::PathBuf};

use crate::{
    game::{control::MapControl, flag::FlagsOwnership},
    map::reader::{MapReader, MapReaderError},
};

use super::{phase::Phase, BattleState};

pub struct BattleStateBuilder<'a> {
    map_name: String,
    resources: &'a PathBuf,
    a_control: &'a MapControl,
    b_control: &'a MapControl,
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

impl<'a> BattleStateBuilder<'a> {
    pub fn new(
        map_name: &str,
        resources: &'a PathBuf,
        a_control: &'a MapControl,
        b_control: &'a MapControl,
    ) -> Self {
        Self {
            map_name: map_name.to_string(),
            resources,
            a_control,
            b_control,
        }
    }

    pub fn build(&self) -> Result<BattleState, BattleStateBuilderError> {
        let map = MapReader::new(&self.map_name, self.resources)?.build()?;
        let flags = FlagsOwnership::from_control(&map, &self.a_control, &self.b_control);
        let mut state =
            BattleState::new(map, vec![], vec![], HashMap::new(), Phase::Placement, flags);
        state.resolve();
        Ok(state)
    }
}
