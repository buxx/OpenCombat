use std::path::{Path, PathBuf};

use crate::indexed::{Indexed, index_all};
use derive_more::Constructor;
use thiserror::Error;

pub const SOUNDS_FOLDER: &str = "sounds";

oc_utils::index_type! {
    pub struct SoundIndex(pub u32);
}

pub type IndexedSound = Indexed<SoundIndex, Sound>;
#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Sound {
    pub name: String,
}

/// Sounds groups as written in mod.ron (sounds file names)
#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Default, PartialEq)]
#[serde(default)]
pub struct SoundGroupsNames {
    pub die: Vec<String>,
    pub main_click: Option<String>,
    pub secondary_click: Option<String>,
    pub deployment_start: Option<String>,
    pub fight_start: Option<String>,
    pub bullet_impact_ground: Vec<String>,
    pub bullet_wall_impact: Vec<String>,
    pub bullet_metal_impact: Vec<String>,
    pub unpin_grenade: Vec<String>,
}

/// Sounds groups resolved to sounds indexes
#[oc_derive::archivable]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SoundGroups {
    pub die: Vec<SoundIndex>,
    pub main_click: Option<SoundIndex>,
    pub secondary_click: Option<SoundIndex>,
    pub deployment_start: Option<SoundIndex>,
    pub fight_start: Option<SoundIndex>,
    pub bullet_impact_ground: Vec<SoundIndex>,
    pub bullet_wall_impact: Vec<SoundIndex>,
    pub bullet_metal_impact: Vec<SoundIndex>,
    pub unpin_grenade: Vec<SoundIndex>,
}

/// Pick randomly one of the given sounds
pub fn pick(sounds: &[SoundIndex]) -> Option<SoundIndex> {
    if sounds.is_empty() {
        return None;
    }
    sounds.get(fastrand::usize(..sounds.len())).copied()
}

pub fn load(path: &Path) -> Result<Vec<IndexedSound>, Error> {
    let path = path.join(SOUNDS_FOLDER);
    let mut sounds = vec![];
    let files = std::fs::read_dir(&path);
    let files = files.map_err(|e| Error::CantReadSoundsFolder(path.clone(), e))?;

    for file in files {
        let file = file.map_err(Error::CantReadSoundsFile)?;
        let file = file.path();
        let name = file.file_name();
        let name = name.ok_or(Error::AbnormalSoundFileName(file.clone()))?;
        let name = name.to_str();
        let name = name.ok_or(Error::AbnormalSoundFileName(file.clone()))?;
        sounds.push(Sound::new(name.to_string()));
    }

    Ok(index_all(sounds))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Can't read sounds folder ({0}): {1}")]
    CantReadSoundsFolder(PathBuf, std::io::Error),
    #[error("Can't read sounds file: {0}")]
    CantReadSoundsFile(std::io::Error),
    #[error("Abnormal sound file: {0}")]
    AbnormalSoundFileName(PathBuf),
}
