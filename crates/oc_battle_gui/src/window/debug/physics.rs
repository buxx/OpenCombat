use derive_more::Constructor;
use oc_geo::region::Region;
use oc_geo::{Geo, region::RegionXy};
use oc_physics::Physic;
use oc_root::geo::WorldVec3;
use oc_root::{WcfgFrom, WcfgInto, WorldConfig};

#[derive(Debug, Clone, Constructor)]
pub struct PhysicsRepr {
    pub position: WorldVec3,
    // pub tile: TileXy,
    pub region: RegionXy,
    // pub forces: Vec<Force>,
}

impl<T: Geo + Physic + Region> WcfgFrom<T> for PhysicsRepr {
    fn from_(value: T, w: &WorldConfig) -> Self {
        Self {
            position: value.position(w),
            region: value.region().into_(w),
        }
    }
}
