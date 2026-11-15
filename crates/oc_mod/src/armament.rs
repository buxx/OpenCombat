use oc_root::physics::Seconds;

use crate::indexed::Indexed;
use crate::{Mod, sound::SoundIndex};

oc_utils::index_type! {
    pub struct ShotModeIndex(pub u32);
}

pub type IndexedShotMode = Indexed<ShotModeIndex, ShotMode>;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub enum ShotMode {
    Single {
        sounds: Vec<SoundIndex>,
    },
    Burst3 {
        interval: Seconds,
        sounds: Vec<SoundIndex>,
    },
    Burst6 {
        interval: Seconds,
        sounds: Vec<SoundIndex>,
    },
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub enum ShotModeRaw {
    Single {
        sounds: Vec<String>,
    },
    Burst3 {
        interval: Seconds,
        sounds: Vec<String>,
    },
    Burst6 {
        interval: Seconds,
        sounds: Vec<String>,
    },
}

impl ShotModeRaw {
    pub fn resolve_(self, mod_: &Mod) -> Result<ShotMode, super::Error> {
        Ok(match self {
            ShotModeRaw::Single { sounds } => ShotMode::Single {
                sounds: mod_
                    .find_sounds(&sounds)?
                    .iter()
                    .map(|s| s.index())
                    .collect(),
            },
            ShotModeRaw::Burst3 { interval, sounds } => ShotMode::Burst3 {
                interval,
                sounds: mod_
                    .find_sounds(&sounds)?
                    .iter()
                    .map(|s| s.index())
                    .collect(),
            },
            ShotModeRaw::Burst6 { interval, sounds } => ShotMode::Burst6 {
                interval,
                sounds: mod_
                    .find_sounds(&sounds)?
                    .iter()
                    .map(|s| s.index())
                    .collect(),
            },
        })
    }
}

impl ShotMode {
    pub fn name(&self) -> &str {
        match self {
            ShotMode::Single { sounds: _ } => "Single",
            ShotMode::Burst3 {
                interval: _,
                sounds: _,
            } => "Burst3",
            ShotMode::Burst6 {
                interval: _,
                sounds: _,
            } => "Burst6",
        }
    }

    pub fn rounds(&self) -> usize {
        match self {
            ShotMode::Single { sounds: _ } => 1,
            ShotMode::Burst3 {
                interval: _,
                sounds: _,
            } => 3,
            ShotMode::Burst6 {
                interval: _,
                sounds: _,
            } => 6,
        }
    }

    pub fn interval(&self) -> Seconds {
        match self {
            ShotMode::Single { sounds: _ } => Seconds(0.),
            ShotMode::Burst3 {
                interval,
                sounds: _,
            } => *interval,
            ShotMode::Burst6 {
                interval,
                sounds: _,
            } => *interval,
        }
    }

    pub fn sounds(&self) -> &Vec<SoundIndex> {
        match self {
            ShotMode::Single { sounds } => sounds,
            ShotMode::Burst3 {
                interval: _,
                sounds,
            } => sounds,
            ShotMode::Burst6 {
                interval: _,
                sounds,
            } => sounds,
        }
    }
}
