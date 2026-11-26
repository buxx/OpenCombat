use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_mod::{Mod, nature::Traversability};
use oc_physics::{Force, IgnoreSide, Physic, collision::Material, volume::Volume};
use oc_root::{
    WcfgFrom, WorldConfig,
    geo::{WorldVec2, WorldVec3},
    material::MaterialKind,
};
use oc_utils::d2::Direction;
use oc_vehicle::Vehicle;

use crate::utils::oriented_rectangle_covered_tiles;

oc_utils::index_type! {
    /// Wreck position in `World::wrecks`
    pub struct WreckIndex(pub u64);
}

/// Wreck left on the ground (kept between battles)
#[oc_derive::archivable(no_compare)]
#[derive(Debug, Clone, PartialEq)]
pub enum Wreck {
    /// Destroyed vehicle chassis
    Vehicle {
        /// Mod vehicle type
        type_: oc_mod::vehicle::VehicleIndex,
        /// Chassis center
        position: WorldVec3,
        chassis_direction: Direction,
    },
}

impl Wreck {
    pub fn from_vehicle(vehicle: &Vehicle) -> Self {
        Self::Vehicle {
            type_: vehicle.type_,
            position: vehicle.position,
            chassis_direction: vehicle.chassis_direction,
        }
    }

    pub fn position(&self) -> WorldVec3 {
        match self {
            Self::Vehicle { position, .. } => *position,
        }
    }

    /// Tile where the wreck center is
    pub fn tile(&self, w: &WorldConfig) -> WorldTileIndex {
        WorldTileIndex::from_(TileXy::from_(WorldVec2::from(self.position()), w), w)
    }

    /// Tiles covered by the wreck (considering chassis size and orientation)
    pub fn covered_tiles(&self, w: &WorldConfig, mod_: &Mod) -> Vec<WorldTileIndex> {
        match self {
            Self::Vehicle {
                type_,
                position,
                chassis_direction,
            } => {
                let (width, length) = mod_.vehicle(*type_).size;
                oriented_rectangle_covered_tiles(
                    [position.x, position.y],
                    width.pixels(w),
                    length.pixels(w),
                    *chassis_direction,
                    w,
                )
            }
        }
    }

    /// Whether the wreck occupies given altitude (pixels)
    pub fn occupies_z(&self, z: f32, w: &WorldConfig, mod_: &Mod) -> bool {
        match self {
            Self::Vehicle {
                type_, position, ..
            } => z >= position.z && z <= position.z + mod_.vehicle(*type_).height.pixels(w),
        }
    }
}

impl Physic for Wreck {
    fn position(&self, _: &WorldConfig) -> WorldVec3 {
        Wreck::position(self)
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        static EMPTY: Vec<Force> = vec![];
        &EMPTY
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        _: bool,
        w: &WorldConfig,
        mod_: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        match self {
            Self::Vehicle {
                type_,
                chassis_direction,
                ..
            } => {
                // Same chassis box than living vehicle (see `oc_vehicle::Vehicle` `Physic`)
                let type_ = mod_.vehicle(*type_);
                let (width, length) = type_.size;
                let cube = Volume::Cube {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                    width: width.pixels(w),
                    height: length.pixels(w),
                    depth: type_.height.pixels(w),
                }
                .with_ref(ref_);

                vec![(
                    cube,
                    Traversability {
                        individual: true,
                        vehicle: false,
                        projectile: false,
                    },
                    *chassis_direction,
                )]
            }
        }
    }

    fn ignore_side(&self) -> IgnoreSide {
        IgnoreSide::None
    }

    fn ignore_underground(&self) -> bool {
        true
    }
}

impl Material for Wreck {
    fn kind(&self) -> Option<MaterialKind> {
        match self {
            Self::Vehicle { .. } => Some(MaterialKind::Vehicle),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mod_() -> Mod {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mods/tests1");
        Mod::load(&root, None).unwrap()
    }

    #[test]
    fn world_wreck_destroyed_vehicles() {
        // Given: one destroyed and one living vehicle
        let mod_ = mod_();
        let w = WorldConfig::new(50, 50);
        let ppt = w.geo_pixels_per_tile() as f32;
        let vehicle = |x: f32, destroyed: bool| {
            let mut vehicle = Vehicle::new(
                oc_root::side::Side::A,
                oc_mod::vehicle::VehicleIndex(0),
                oc_mod::illustration::IllustrationIndex(0),
                // FIXME BS NOW: 25.5 ?!
                WorldVec3::new(x * ppt, 25.5 * ppt, 0.),
                vec![],
                Direction::EST,
                Some(Direction::NORTH),
            );
            vehicle.destroyed = destroyed;
            vehicle
        };
        // FIXME BS NOW: 10.5 ?!
        let destroyed = vehicle(10.5, true);
        let nature = oc_mod::nature::NatureIndex(0);
        let allow = mod_.nature(nature).traversability.clone();
        let tiles = (0..w.tiles_count())
            .map(|i| crate::tile::Tile::new(WorldTileIndex(i), nature, 0, allow.clone()))
            .collect();
        let snapshot = crate::snapshot::Snapshot::empty(w.clone())
            .with_tiles(tiles)
            .with_phase(oc_root::battle::BattlePhase::Fight)
            // FIXME BS NOW: 40.5 ?!
            .with_vehicles(vec![destroyed.clone(), vehicle(40.5, false)]);
        let meta = crate::meta::Meta::default();
        let mut world = crate::World::from_snapshot(mod_, meta, &Default::default(), snapshot);

        // When
        world.wreck_destroyed_vehicles();

        // Then: only destroyed vehicle is a wreck, keeping its chassis direction
        let wreck = Wreck::from_vehicle(&destroyed);
        assert_eq!(world.wrecks, vec![wreck.clone()]);
        assert!(matches!(
            wreck,
            Wreck::Vehicle { chassis_direction, .. } if chassis_direction == Direction::EST
        ));
    }
}
