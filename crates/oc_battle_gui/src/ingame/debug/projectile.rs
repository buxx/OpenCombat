use std::time::Duration;

use derive_more::Constructor;
use oc_mod::{ammunition::AmmunitionIndex, armament::ShotModeIndex, weapons::WeaponIndex};
use oc_root::{physics::Meters, side::Side};

#[derive(Debug, Clone, Constructor)]
pub struct SpawnProjectileProfile {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
    pub repeat: u8,
    pub plus_z: Meters,
    pub side: Side,
    pub lag: Duration,
}
