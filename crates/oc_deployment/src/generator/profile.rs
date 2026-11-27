use std::borrow::Cow;

use oc_mod::{
    Mod,
    squad::{SquadPreset, default_count},
};
use serde::{Deserialize, Serialize};

pub use oc_mod::squad::{Individual, Magazine, Shells, Weapons};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub squads: Vec<Squad>,
}

/// Squads to deploy: a mod preset (declared in mod `squads.ron`) or a squad declared here
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Squad {
    /// Mod squad preset, by its label, like `Preset(label: "Rifle section", count: 4)`
    Preset {
        label: String,
        #[serde(default = "default_count")]
        count: u32,
    },
    /// Squad fully declared, like `Custom(count: 2, label: "Riffle", illustration: ..)`
    Custom {
        #[serde(default = "default_count")]
        count: u32,
        label: String,
        illustration: String,
        individuals: Vec<Individual>,
        #[serde(default)]
        vehicle: Option<String>,
        /// Shells stored in the squad vehicle (ignored when no vehicle)
        #[serde(default)]
        shells: Vec<Shells>,
    },
}

impl Squad {
    pub fn count(&self) -> u32 {
        match self {
            Squad::Preset { count, .. } | Squad::Custom { count, .. } => *count,
        }
    }

    /// Squad composition (from `mod_` presets when it is a preset)
    pub fn resolve<'a>(&'a self, mod_: &'a Mod) -> Result<Cow<'a, SquadPreset>, oc_mod::Error> {
        match self {
            Squad::Preset { label, .. } => Ok(Cow::Borrowed(mod_.squad_preset(label)?)),
            Squad::Custom {
                count: _,
                label,
                illustration,
                individuals,
                vehicle,
                shells,
            } => Ok(Cow::Owned(SquadPreset {
                label: label.clone(),
                illustration: illustration.clone(),
                individuals: individuals.clone(),
                vehicle: vehicle.clone(),
                shells: shells.clone(),
            })),
        }
    }
}
