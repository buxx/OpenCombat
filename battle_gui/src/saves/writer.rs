use battle_core::state::battle::BattleState;
use std::fs::File;
use std::io::prelude::*;
use std::{io, path::PathBuf};
use thiserror::Error;

pub struct BattleStateWriter {
    destination: PathBuf,
}

impl BattleStateWriter {
    pub fn new(destination: PathBuf) -> Self {
        Self { destination }
    }

    pub fn write(&self, battle_state: &BattleState) -> Result<(), BattleStateWriterError> {
        let mut file = File::create(&self.destination)?;
        // TODO : use firsts bytes to write save game version and manage possible upgrades
        file.write_all(&bincode::serialize(&battle_state.copy())?)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum BattleStateWriterError {
    #[error("Disk error : {0}")]
    Disk(io::Error),
    #[error("Serialization error : {0}")]
    Serialization(bincode::ErrorKind),
}

impl From<io::Error> for BattleStateWriterError {
    fn from(value: io::Error) -> Self {
        Self::Disk(value)
    }
}

impl From<Box<bincode::ErrorKind>> for BattleStateWriterError {
    fn from(value: Box<bincode::ErrorKind>) -> Self {
        Self::Serialization(*value)
    }
}
