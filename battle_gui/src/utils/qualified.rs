use std::path::PathBuf;

use ggez::GameError;
use oc_core::resources::Resources;
use thiserror::Error;

use crate::graphics::{map::ensure_dark, qualified::Zoom};

pub trait ToQualified<T, E> {
    fn to_qualified(&self, zoom: &Zoom) -> Result<T, E>;
    fn to_dark(&self, map_name: &str) -> Result<T, E>;
}

impl ToQualified<PathBuf, PathQualificationError> for PathBuf {
    fn to_qualified(&self, zoom: &Zoom) -> Result<PathBuf, PathQualificationError> {
        Ok(self
            .parent()
            .ok_or_else(|| PathQualificationError::NoParent(self.clone()))?
            .join(format!(
                "{}{}.png",
                self.file_stem()
                    .ok_or_else(|| PathQualificationError::NoStem(self.clone()))?
                    .to_str()
                    .ok_or_else(|| PathQualificationError::Unexpected(
                        self.display().to_string()
                    ))?,
                zoom.suffix()
            )))
    }

    fn to_dark(&self, map_name: &str) -> Result<PathBuf, PathQualificationError> {
        ensure_dark(map_name, &self)?;
        let resources = match Resources::new() {
            Ok(resources) => resources,
            Err(error) => return Err(PathQualificationError::Unexpected(error.to_string())),
        };
        Ok(resources.cache_ggez().join(format!(
            "{}__{}{}.png",
            map_name,
            self.file_stem()
                .ok_or_else(|| PathQualificationError::NoStem(self.clone()))?
                .to_str()
                .ok_or_else(|| PathQualificationError::Unexpected(self.display().to_string()))?,
            "__dark"
        )))
    }
}

#[derive(Error, Debug)]
pub enum PathQualificationError {
    #[error("Path have no parent : {0}")]
    NoParent(PathBuf),
    #[error("Unexpected to extract non extension file name from : {0}")]
    NoStem(PathBuf),
    #[error("Unexpected format : {0}")]
    Unexpected(String),
}

impl From<GameError> for PathQualificationError {
    fn from(value: GameError) -> Self {
        Self::Unexpected(value.to_string())
    }
}
