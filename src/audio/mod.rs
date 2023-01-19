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
    MaleScreaming1,
    MaleScreaming2,
    MaleScreaming3,
    MaleScreaming4,
    MaleScreaming5,
    MaleDie1,
    MaleDie2,
    MaleDie3,
    MaleDie4,
    MaleDie5,
    MaleDie6,
    MaleDie7,
    MaleDie8,
    MetalHit1,
    Bip1,
    Clac1,
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
            Sound::MaleScreaming1 => "/audio/MaleScreaming1.ogg",
            Sound::MaleScreaming2 => "/audio/MaleScreaming2.ogg",
            Sound::MaleScreaming3 => "/audio/MaleScreaming3.ogg",
            Sound::MaleScreaming4 => "/audio/MaleScreaming4.ogg",
            Sound::MaleScreaming5 => "/audio/MaleScreaming5.ogg",
            Sound::MaleDie1 => "/audio/MaleDie1.ogg",
            Sound::MaleDie2 => "/audio/MaleDie2.ogg",
            Sound::MaleDie3 => "/audio/MaleDie3.ogg",
            Sound::MaleDie4 => "/audio/MaleDie4.ogg",
            Sound::MaleDie5 => "/audio/MaleDie5.ogg",
            Sound::MaleDie6 => "/audio/MaleDie6.ogg",
            Sound::MaleDie7 => "/audio/MaleDie7.ogg",
            Sound::MaleDie8 => "/audio/MaleDie8.ogg",
            Sound::MetalHit1 => "/audio/MetalHit1.ogg",
            Sound::Bip1 => "/audio/Bip1.ogg",
            Sound::Clac1 => "/audio/Clac1.ogg",
        }
        .to_string()
    }
}
