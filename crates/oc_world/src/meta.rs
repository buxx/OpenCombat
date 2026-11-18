use std::path::Path;

use derive_more::Constructor;
use oc_root::format::{FileError, read_toml};

#[oc_derive::archivable(serde)]
#[derive(Debug, Constructor, Clone, PartialEq)]
pub struct Meta {
    #[serde(skip, default)]
    pub name: String,
    pub revision: u32,
    pub geo_meters_per_z: f32,
    #[serde(default)]
    pub address: Option<String>,
}

#[cfg(feature = "tests")]
impl Default for Meta {
    fn default() -> Self {
        Self {
            name: "TestMeta".to_string(),
            revision: 0,
            geo_meters_per_z: 0.1,
            address: None,
        }
    }
}

impl Meta {
    pub fn from_file(path: &Path) -> Result<Self, FileError> {
        // TODO
        let name = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let mut meta: Self = read_toml(path)?;
        meta.name = name;
        Ok(meta)
    }

    pub fn canonical(&self) -> String {
        format!("{}_{}", self.name, self.revision)
    }

    pub fn archive(&self) -> String {
        format!("{}.tar.gz", self.canonical())
    }
}
