use std::{env::current_exe, path::PathBuf};

use anyhow::{Context, Error, Result};

pub struct BattleLauncher {
    executable_path: PathBuf,
}

impl BattleLauncher {
    pub fn new() -> Result<Self> {
        let executable_path = current_exe().context("Retrieve current executable path")?;
        Ok(Self { executable_path })
    }

    pub fn launch(&self) -> Result<()> {
        Err(Error::msg("foo"))
    }
}
