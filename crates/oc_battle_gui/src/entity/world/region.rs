use bevy::prelude::*;
#[cfg(feature = "debug")]
use oc_geo::region::RegionXy;

#[derive(Debug, Component)]
pub struct RegionBackground;

#[cfg(feature = "debug")]
#[derive(Debug, Component)]
pub struct RegionWireFrame(pub RegionXy);
