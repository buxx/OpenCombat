use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::indexed::{Indexed, index_all};
use anyhow::Context;
use derive_more::Constructor;
use strum_macros::Display;
use thiserror::Error;

pub const ILLUSTRATION_RON: &str = "illustrations.ron";

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum IllustrationKind {
    IngameSquad,
    IngameIndividual,
}

oc_utils::index_type! {
    pub struct IllustrationIndex(pub u32);
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteSheet {
    pub start_x: u32,
    pub start_y: u32,
    pub columns: u32,
    pub sprite_width: u32,
    pub sprite_height: u32,
    pub sprites: Vec<String>,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct RawIllustrations {
    pub ingame_squads: SpriteSheet,
    pub ingame_individuals: SpriteSheet,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IndexedIllustrations {
    pub ingame_squads: Vec<IndexedIllustration>,
    pub ingame_individuals: Vec<IndexedIllustration>,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Illustration {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub type IndexedIllustration = Indexed<IllustrationIndex, Illustration>;

pub fn load(path: &Path) -> Result<IndexedIllustrations, Error> {
    let path = path.join(ILLUSTRATION_RON);
    if !path.exists() {
        return Err(Error::NotFound(path));
    }

    let raw = std::fs::read_to_string(&path).context(format!("Read {}", path.display()))?;
    let raw: RawIllustrations = ron::from_str(&raw)?;

    if !raw.ingame_squads.sprites.iter().any(|n| n == "default") {
        return Err(Error::NoDefault(IllustrationKind::IngameSquad));
    }
    if !raw
        .ingame_individuals
        .sprites
        .iter()
        .any(|n| n == "default")
    {
        return Err(Error::NoDefault(IllustrationKind::IngameSquad));
    }

    let squads = index_sheet(IllustrationKind::IngameSquad, raw.ingame_squads)?;
    let individuals = index_sheet(IllustrationKind::IngameIndividual, raw.ingame_individuals)?;

    Ok(IndexedIllustrations {
        ingame_squads: squads,
        ingame_individuals: individuals,
    })
}

fn index_sheet(
    kind: IllustrationKind,
    sheet: SpriteSheet,
) -> Result<Vec<IndexedIllustration>, Error> {
    let mut illustrations = vec![];
    let mut names = HashSet::new();

    if sheet.columns == 0 {
        return Err(Error::NoColumns(kind));
    }

    for (i, name) in sheet.sprites.iter().enumerate() {
        if !names.insert(name.clone()) {
            return Err(Error::DuplicateName(name.clone()));
        }

        let i = i as u32;
        illustrations.push(Illustration::new(
            name.clone(),
            sheet.start_x + (i % sheet.columns) * sheet.sprite_width,
            sheet.start_y + (i / sheet.columns) * sheet.sprite_height,
            sheet.sprite_width,
            sheet.sprite_height,
        ));
    }

    Ok(index_all(illustrations))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("No default sprite for {0}")]
    NoDefault(IllustrationKind),
    #[error("No illustration file ({0}) found")]
    NotFound(PathBuf),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("SpriteSheet {0}: columns must be greater than 0")]
    NoColumns(IllustrationKind),
    #[error("Duplicate illustration name: {0}")]
    DuplicateName(String),
}
