use std::path::PathBuf;

use thiserror::Error;

use crate::graphics::qualified::Zoom;

pub trait ToQualified<T, E> {
    fn to_qualified(&self, zoom: &Zoom) -> Result<T, E>;
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
                    .ok_or_else(|| PathQualificationError::Unexpected(self.clone()))?,
                zoom.suffix()
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
    Unexpected(PathBuf),
}
