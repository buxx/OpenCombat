use std::path::Path;

use crate::{
    Mod,
    indexed::{ArchivedDebug, Indexed, index_all, read_ron},
    sound::SoundIndex,
};
use oc_root::{explosion::Explosion, physics::Meters};
use thiserror::Error;

pub const GRENADES_RON: &str = "grenades.ron";

oc_utils::index_type! {
    pub struct GrenadeIndex(pub u16);
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrenadeNature {
    Frag,
    Smoke,
}

pub type IndexedGrenade = Indexed<GrenadeIndex, Grenade>;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Grenade<S: ArchivedDebug = Vec<SoundIndex>> {
    pub name: String,
    pub blast_rayon: Meters,
    pub shrapnel_rayon: Meters,
    #[serde(default)]
    pub suppress_rayon: Meters,
    pub shrapnel_quantity: u32,
    pub nature: GrenadeNature,
    #[serde(default)]
    pub sounds: S,
    /// Smoke only: seconds the smoke keeps its full opacity
    #[serde(default)]
    pub smoke_duration: f32,
    /// Smoke only: seconds, after `smoke_duration`, the smoke takes to disappear
    #[serde(default)]
    pub smoke_fade_duration: f32,
}

/// Grenade as written in grenades.ron (sounds referenced by name)
pub type GrenadeRaw = Grenade<Vec<String>>;

impl Grenade {
    pub fn pick_sound(&self) -> Option<SoundIndex> {
        crate::sound::pick(&self.sounds)
    }
}

impl From<Grenade> for Explosion {
    fn from(value: Grenade) -> Self {
        match value.nature {
            GrenadeNature::Frag => Self {
                kill_rayon: value.blast_rayon,
                stunned_rayon: value.blast_rayon,
                shrapnel_rayon: value.shrapnel_rayon,
                suppress_rayon: value.suppress_rayon,
                destruction_rayon: Meters(0.0),
                destruction_force: 0.0,
                shrapnel_quantity: value.shrapnel_quantity,
                penetration_force: 0.0,
            },
            GrenadeNature::Smoke => Self {
                kill_rayon: Meters(0.0),
                stunned_rayon: Meters(0.0),
                shrapnel_rayon: Meters(0.0),
                suppress_rayon: Meters(0.0),
                destruction_rayon: Meters(0.0),
                destruction_force: 0.0,
                shrapnel_quantity: 0,
                penetration_force: 0.0,
            },
        }
    }
}

impl GrenadeRaw {
    fn resolve(self, mod_: &Mod) -> Result<Grenade, Error> {
        let sounds = mod_.find_sound_indexes(&self.sounds).map_err(Box::new)?;
        Ok(Grenade {
            name: self.name,
            blast_rayon: self.blast_rayon,
            shrapnel_rayon: self.shrapnel_rayon,
            suppress_rayon: self.suppress_rayon,
            shrapnel_quantity: self.shrapnel_quantity,
            nature: self.nature,
            sounds,
            smoke_duration: self.smoke_duration,
            smoke_fade_duration: self.smoke_fade_duration,
        })
    }
}

pub fn load(path: &Path, mod_: &Mod) -> Result<Vec<IndexedGrenade>, Error> {
    let grenades: Vec<GrenadeRaw> = read_ron::<_, Error>(path, GRENADES_RON)?;
    Ok(index_all(
        grenades
            .into_iter()
            .map(|grenade| grenade.resolve(mod_))
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("{0}")]
    Mod(#[from] Box<crate::Error>),
}
