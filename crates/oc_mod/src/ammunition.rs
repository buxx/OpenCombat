use std::path::Path;

use crate::indexed::{Indexed, index_all, read_ron};
use strum_macros::EnumIter;
use thiserror::Error;

pub const PROJECTILES_RON: &str = "amunitions.ron";

oc_utils::index_type! {
    pub struct AmmunitionIndex(pub u32);
}

pub type IndexedAmmunition = Indexed<AmmunitionIndex, Ammunition>;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ammunition {
    Cartridge(Cartridge),
}

impl Ammunition {
    pub fn name(&self) -> &str {
        match self {
            Ammunition::Cartridge(bullet) => &bullet.name,
        }
    }

    pub fn is_type(&self, type_: AmmunitionType) -> bool {
        match self {
            Ammunition::Cartridge(_) => matches!(type_, AmmunitionType::Cartridge),
        }
    }

    pub fn model(&self) -> &String {
        match self {
            Ammunition::Cartridge(cartridge) => &cartridge.model,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Default)]
pub enum AmmunitionType {
    #[default]
    Cartridge,
}

impl AmmunitionType {
    pub fn name(&self) -> &str {
        match self {
            AmmunitionType::Cartridge => "Cartridge",
        }
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cartridge {
    name: String,
    model: String,
}

pub fn load(path: &Path) -> Result<Vec<IndexedAmmunition>, Error> {
    let amunitions: Vec<Ammunition> = read_ron::<_, Error>(path, PROJECTILES_RON)?;

    if amunitions.is_empty() {
        return Err(Error::Empty);
    }

    Ok(index_all(amunitions))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("No amunitions defined (require at least one)")]
    Empty,
}
