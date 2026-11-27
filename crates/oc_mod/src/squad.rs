use std::{collections::HashSet, path::Path};

use thiserror::Error;

use crate::indexed::read_ron;

pub const SQUADS_RON: &str = "squads.ron";

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct SquadPreset {
    pub label: String,
    pub illustration: String,
    pub individuals: Vec<Individual>,
    #[serde(default)]
    pub vehicle: Option<String>,
    #[serde(default)]
    pub shells: Vec<Shells>,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Shells {
    pub name: String,
    pub count: u32,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Individual {
    #[serde(default = "default_count")]
    pub count: u32,
    pub illustration: String,
    #[serde(default = "default_xp")]
    pub xp: [u8; 2],
    pub weapons: Weapons,
    pub magazines: Vec<Magazine>,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Weapons {
    pub main: Option<String>,
    #[serde(default)]
    pub grenades: Vec<String>,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Magazine {
    #[serde(default = "default_count")]
    pub count: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ammunition: Option<String>,
}

pub fn default_count() -> u32 {
    1
}

fn default_xp() -> [u8; 2] {
    [128, 75]
}

pub fn load(path: &Path) -> Result<Vec<SquadPreset>, Error> {
    if !path.join(SQUADS_RON).exists() {
        return Ok(vec![]);
    }
    let squads: Vec<SquadPreset> = read_ron::<_, Error>(path, SQUADS_RON)?;

    let mut labels = HashSet::new();
    if let Some(squad) = squads.iter().find(|s| !labels.insert(s.label.as_str())) {
        return Err(Error::DuplicateLabel(squad.label.clone()));
    }

    Ok(squads)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] oc_root::format::FileError),
    #[error("Duplicate squad preset label: {0}")]
    DuplicateLabel(String),
}
