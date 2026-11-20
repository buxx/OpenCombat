// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use derive_more::Constructor;
use oc_mod::illustration::IllustrationIndex;
use oc_physics::Force;
use oc_root::{geo::WorldVec3, side::Side};
use oc_utils::d2::Direction;

pub mod network;

oc_utils::index_type! {
    pub struct VehicleIndex(pub u64);
}

#[oc_derive::archivable]
#[derive(Debug, PartialEq, Constructor, Clone)]
pub struct Vehicle {
    pub side: Side,
    pub type_: oc_mod::vehicle::VehicleIndex,
    pub illustration: IllustrationIndex,
    pub position: WorldVec3,
    pub forces: Vec<Force>,
    pub chassis_orientation: Direction,
    pub turret_direction: Option<Direction>,
}
