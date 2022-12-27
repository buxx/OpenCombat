use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

pub mod player;

#[derive(Debug, Hash, Copy, Serialize, Deserialize, Clone, EnumIter, Eq, PartialEq)]
pub enum Sound {
    MosinNagantFire1,
    MosinNagantFire2,
    MosinNagantFire3,
    MosinNagantFire4,
    MosinNagantFire5,
    MosinNagantReload1,
    MosinNagantReload2,
    MosinNagantReload3,
    MosinNagantReload4,
    CannonFire1,
}

impl Sound {
    pub fn file_path(&self) -> String {
        match self {
            Sound::MosinNagantFire1 => "/audio/MosinNagantFire1.ogg",
            Sound::MosinNagantFire2 => "/audio/MosinNagantFire2.ogg",
            Sound::MosinNagantFire3 => "/audio/MosinNagantFire3.ogg",
            Sound::MosinNagantFire4 => "/audio/MosinNagantFire4.ogg",
            Sound::MosinNagantFire5 => "/audio/MosinNagantFire5.ogg",
            Sound::MosinNagantReload1 => "/audio/MosinNagantReload1.ogg",
            Sound::MosinNagantReload2 => "/audio/MosinNagantReload2.ogg",
            Sound::MosinNagantReload3 => "/audio/MosinNagantReload3.ogg",
            Sound::MosinNagantReload4 => "/audio/MosinNagantReload4.ogg",
            Sound::CannonFire1 => "/audio/CannonFire1.ogg",
        }
        .to_string()
    }
}
