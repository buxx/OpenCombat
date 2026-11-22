use std::path::Path;

use oc_root::{
    explosion::Explosion,
    physics::{Meters, MetersSeconds},
};

use crate::{
    Mod,
    indexed::{ArchivedDebug, Indexed, index_all, read_ron},
    sound::SoundIndex,
};
use thiserror::Error;

pub const SHELLS_RON: &str = "shells.ron";

oc_utils::index_type! {
    pub struct ShellIndex(pub u16);
}

pub type IndexedShell = Indexed<ShellIndex, Shell>;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Shell<S: ArchivedDebug = Vec<SoundIndex>> {
    pub name: String,
    /// Shell flight speed (when fired by a gun)
    pub velocity: MetersSeconds,
    pub kill_rayon: Meters,
    pub stunned_rayon: Meters,
    pub shrapnel_rayon: Meters,
    #[serde(default)]
    pub suppress_rayon: Meters,
    pub destruction_rayon: Meters,
    pub destruction_force: f32,
    pub shrapnel_quantity: u32,
    pub penetration_force: f32,
    #[serde(default)]
    pub sounds: S,
}

/// Shell as written in shells.ron (sounds referenced by name)
pub type ShellRaw = Shell<Vec<String>>;

impl Shell {
    pub fn pick_sound(&self) -> Option<SoundIndex> {
        crate::sound::pick(&self.sounds)
    }
}

impl From<Shell> for Explosion {
    fn from(value: Shell) -> Self {
        Self {
            kill_rayon: value.kill_rayon,
            stunned_rayon: value.stunned_rayon,
            shrapnel_rayon: value.shrapnel_rayon,
            suppress_rayon: value.suppress_rayon,
            destruction_rayon: value.destruction_rayon,
            destruction_force: value.destruction_force,
            shrapnel_quantity: value.shrapnel_quantity,
            penetration_force: value.penetration_force,
        }
    }
}

impl ShellRaw {
    fn resolve(self, mod_: &Mod) -> Result<Shell, Error> {
        let sounds = mod_.find_sound_indexes(&self.sounds).map_err(Box::new)?;
        Ok(Shell {
            name: self.name,
            velocity: self.velocity,
            kill_rayon: self.kill_rayon,
            stunned_rayon: self.stunned_rayon,
            shrapnel_rayon: self.shrapnel_rayon,
            suppress_rayon: self.suppress_rayon,
            destruction_rayon: self.destruction_rayon,
            destruction_force: self.destruction_force,
            shrapnel_quantity: self.shrapnel_quantity,
            penetration_force: self.penetration_force,
            sounds,
        })
    }
}

pub fn load(path: &Path, mod_: &Mod) -> Result<Vec<IndexedShell>, Error> {
    let shells: Vec<ShellRaw> = read_ron::<_, Error>(path, SHELLS_RON)?;
    Ok(index_all(
        shells
            .into_iter()
            .map(|shell| shell.resolve(mod_))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] oc_root::format::FileError),
    #[error("{0}")]
    Mod(#[from] Box<crate::Error>),
}
