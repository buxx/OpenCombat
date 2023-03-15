use std::fmt::Display;

use battle_core::game::explosive::ExplosiveType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DebugTerrain {
    None,
    Tiles,
    Opacity,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum DebugPhysics {
    None,
    MosinNagantM1924GunFire,
    BrandtMle2731Shelling,
}

impl Display for DebugPhysics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugPhysics::None => f.write_str("Normal"),
            DebugPhysics::MosinNagantM1924GunFire => f.write_str("GunFire (MosinNagantM1924)"),
            DebugPhysics::BrandtMle2731Shelling => f.write_str("Shelling (BrandtMle2731Shelling)"),
        }
    }
}

impl DebugPhysics {
    /// Give arbitrary chosen explosive for
    pub fn explosive(&self) -> Option<ExplosiveType> {
        match self {
            DebugPhysics::None => None,
            DebugPhysics::MosinNagantM1924GunFire => None,
            DebugPhysics::BrandtMle2731Shelling => Some(ExplosiveType::FA19241927),
        }
    }
}
