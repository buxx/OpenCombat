use std::{collections::HashSet, path::Path, time::Duration};

use oc_root::physics::Meters;
use thiserror::Error;

pub const SUPPORT_RON: &str = "support.ron";

use crate::{indexed::ArchivedDebug, indexed::read_ron, shell::ShellIndex};

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Support<S: ArchivedDebug = ShellIndex> {
    pub name: String,
    pub shell: S,
    pub series: u32,
    pub shells_by_serie: u32,
    pub radius: Meters,
    pub prepare_seconds: u64,
    pub serie_interval_ms: u64,
    pub shell_interval_ms: u64,
    pub between_wait_s: u64,
}

pub type SupportRaw = Support<String>;

impl Support {
    pub fn prepare(&self) -> Duration {
        Duration::from_secs(self.prepare_seconds)
    }

    pub fn serie_interval(&self) -> Duration {
        Duration::from_millis(self.serie_interval_ms)
    }

    pub fn between_wait(&self) -> Duration {
        Duration::from_secs(self.between_wait_s)
    }

    pub fn shell_interval(&self) -> Duration {
        Duration::from_millis(self.shell_interval_ms)
    }
}

pub fn load(path: &Path) -> Result<Vec<SupportRaw>, Error> {
    let supports: Vec<SupportRaw> = read_ron::<_, Error>(path, SUPPORT_RON)?;

    let mut names = HashSet::new();
    if let Some(support) = supports.iter().find(|s| !names.insert(s.name.as_str())) {
        return Err(Error::DuplicateName(support.name.clone()));
    }

    Ok(supports)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] oc_root::format::FileError),
    #[error("Duplicate support name: {0}")]
    DuplicateName(String),
}
