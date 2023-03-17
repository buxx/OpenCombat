use std::{fs, path::PathBuf};
use thiserror::Error;

pub const RESOURCE_PATH: &'static str = "./resources";

pub struct Resources {
    home: PathBuf,
}

#[derive(Error, Debug)]
pub enum ResourcesError {
    #[error("Error during determination of home directory")]
    HomeDir,
    #[error("Unable to create directory {0} : {1}")]
    MkDir(PathBuf, String),
}

impl Resources {
    pub fn new() -> Result<Self, ResourcesError> {
        let home = match dirs::home_dir() {
            Some(home) => home,
            None => return Err(ResourcesError::HomeDir),
        };
        Ok(Self { home })
    }

    pub fn ensure(self) -> Result<Self, ResourcesError> {
        for path in &vec![self.cache_abs()] {
            match fs::create_dir_all(path) {
                Err(error) => return Err(ResourcesError::MkDir(path.clone(), error.to_string())),
                _ => {}
            }
        }

        Ok(self)
    }

    pub fn app_abs(&self) -> PathBuf {
        #[cfg(target_os = "linux")]
        {
            self.home.join("Games").join("OpenCombat")
        }

        #[cfg(target_os = "windows")]
        {
            self.home.join("AppData").join("Local").join("OpenCombat")
        }

        #[cfg(target_os = "macos")]
        {
            self.home.join("Library").join("OpenCombat")
        }
    }

    pub fn cache_abs(&self) -> PathBuf {
        self.app_abs().join(self.cache_rel())
    }

    pub fn cache_rel(&self) -> PathBuf {
        PathBuf::from("Cache")
    }

    pub fn cache_ggez(&self) -> PathBuf {
        PathBuf::from("/Cache")
    }

    pub fn lib(&self) -> PathBuf {
        PathBuf::from(RESOURCE_PATH)
    }

    pub fn resources_paths_abs(&self) -> Vec<PathBuf> {
        vec![self.lib(), self.app_abs()]
    }
}
