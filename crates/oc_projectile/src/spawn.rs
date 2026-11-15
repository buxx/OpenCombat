use std::time::Duration;

use derive_more::Constructor;
use oc_individual::IndividualIndex;
use oc_mod::{ammunition::AmmunitionIndex, armament::ShotModeIndex, weapons::WeaponIndex};
use oc_root::{geo::WorldVec3, side::Side};

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq, Constructor)]
#[allow(clippy::too_many_arguments)]
pub struct SpawnProjectiles {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
    pub repeat: u8,
    pub from: WorldVec3,
    pub directions: Vec<WorldVec3>,
    pub side: Side,
    pub lag: Duration,
    pub shooter: Option<IndividualIndex>,
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct SpawnProjectile {
    pub weapon: WeaponIndex,
    pub ammunition: AmmunitionIndex,
    pub shot: ShotModeIndex,
    pub from: WorldVec3,
    pub direction: WorldVec3,
    pub side: Side,
    pub shooter: Option<IndividualIndex>,
}

impl SpawnProjectiles {
    pub fn from_spawns(spawns: &SpawnProjectiles, direction: WorldVec3) -> SpawnProjectile {
        SpawnProjectile {
            weapon: spawns.weapon,
            ammunition: spawns.ammunition,
            shot: spawns.shot,
            from: spawns.from,
            direction,
            side: spawns.side,
            shooter: spawns.shooter,
        }
    }
}
