use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::engine::input::Control;

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
    pub fn next(&self) -> Self {
        match self {
            DebugPhysics::None => DebugPhysics::MosinNagantM1924GunFire,
            DebugPhysics::MosinNagantM1924GunFire => DebugPhysics::BrandtMle2731Shelling,
            DebugPhysics::BrandtMle2731Shelling => DebugPhysics::None,
        }
    }

    pub fn control(&self) -> Control {
        match self {
            DebugPhysics::None => Control::Soldiers,
            _ => Control::Physics,
        }
    }
}
