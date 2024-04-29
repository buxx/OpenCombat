use std::{fs, io, path::PathBuf};

use oc_core::resources::{Resources, ResourcesError};
use thiserror::Error;

pub struct BattleSavesListBuilder<'a> {
    map_name: &'a str,
}

impl<'a> BattleSavesListBuilder<'a> {
    pub fn new(map_name: &'a str) -> Self {
        Self { map_name }
    }

    pub fn build(&self) -> Result<Vec<PathBuf>, BattleSavesListBuilderError> {
        let mut saves = vec![];

        for file in (fs::read_dir(Resources::new()?.battle_saves_abs(self.map_name))?).flatten() {
            saves.push(file.path())
        }

        Ok(saves)
    }
}

#[derive(Error, Debug)]
pub enum BattleSavesListBuilderError {
    #[error("Resource error : {0}")]
    Resource(ResourcesError),
    #[error("Disk error : {0}")]
    Disk(io::Error),
}

impl From<ResourcesError> for BattleSavesListBuilderError {
    fn from(value: ResourcesError) -> Self {
        Self::Resource(value)
    }
}

impl From<io::Error> for BattleSavesListBuilderError {
    fn from(value: io::Error) -> Self {
        Self::Disk(value)
    }
}
