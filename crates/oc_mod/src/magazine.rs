use std::path::Path;

use crate::indexed::{Indexed, index_all, read_ron};
use strum_macros::EnumIter;
use thiserror::Error;

pub const MAGAZINES_RON: &str = "magazine.ron";

oc_utils::index_type! {
    pub struct MagazineIndex(pub u32);
}

pub type IndexedMagazine = Indexed<MagazineIndex, Magazine>;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub enum Magazine {
    Cartridge(Cartridge),
}

impl Magazine {
    pub fn name(&self) -> &str {
        match self {
            Magazine::Cartridge(cartridge) => &cartridge.name,
        }
    }

    pub fn is_type(&self, type_: MagazineType) -> bool {
        match self {
            Magazine::Cartridge(_) => matches!(type_, MagazineType::Cartridge),
        }
    }

    pub fn capacity(&self) -> u16 {
        match self {
            Magazine::Cartridge(cartridge) => cartridge.capacity,
        }
    }

    pub fn accept(&self) -> &Vec<String> {
        match self {
            Magazine::Cartridge(cartridge) => &cartridge.accept,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Default)]
pub enum MagazineType {
    #[default]
    Cartridge,
}

impl MagazineType {
    pub fn name(&self) -> &str {
        match self {
            MagazineType::Cartridge => "Cartridge",
        }
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Cartridge {
    name: String,
    capacity: u16,
    accept: Vec<String>,
}

pub fn load(path: &Path) -> Result<Vec<IndexedMagazine>, Error> {
    let magazines: Vec<Magazine> = read_ron::<_, Error>(path, MAGAZINES_RON)?;

    if magazines.is_empty() {
        return Err(Error::Empty);
    }

    Ok(index_all(magazines))
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
