use std::{path::Path, time::Duration};

use oc_root::physics::Meters;
use thiserror::Error;

pub const SUPPORT_RON: &str = "support.ron";

use crate::{indexed::ArchivedDebug, indexed::read_ron, shell::ShellIndex};

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Support<S: ArchivedDebug = ShellIndex> {
    pub naval_bombard_shell: S,
    pub naval_bombard_series: u32,
    pub naval_bombard_shells_by_serie: u32,
    pub naval_bombard_radius: Meters,
    pub naval_bombard_prepare_seconds: u64,
    pub naval_bombard_serie_interval_ms: u64,
    pub naval_bombard_shell_interval_ms: u64,
    pub naval_bombard_between_wait_s: u64,
}

/// Support as written in support.ron (shell referenced by name)
pub type SupportRaw = Support<String>;

impl Support {
    pub fn naval_bombard_prepare(&self) -> Duration {
        Duration::from_secs(self.naval_bombard_prepare_seconds)
    }

    pub fn naval_bombard_serie_interval(&self) -> Duration {
        Duration::from_millis(self.naval_bombard_serie_interval_ms)
    }

    pub fn naval_bombard_between_wait(&self) -> Duration {
        Duration::from_secs(self.naval_bombard_between_wait_s)
    }

    pub fn naval_bombard_shell_interval(&self) -> Duration {
        Duration::from_millis(self.naval_bombard_shell_interval_ms)
    }
}

pub fn load(path: &Path) -> Result<SupportRaw, Error> {
    read_ron::<_, Error>(path, SUPPORT_RON)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] oc_root::format::FileError),
}
