use std::f32;

use oc_geo::{
    region::WorldRegionIndex,
    tile::{TileXy, WorldTileIndex},
};
use oc_mod::nature::Traversability;
use oc_mod::{Mod, nature::NatureIndex};
use oc_physics::{Force, IgnoreSide, Physic, collision::Material, volume::Volume};
use oc_root::{WcfgInto, WorldConfig, geo::WorldVec3, material::MaterialKind};
use oc_utils::d2::Direction;

use crate::{World, navmesh::Walls};
use derive_more::Constructor;

const DEPTH: f32 = 10_000.;

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Tile {
    pub i: WorldTileIndex, // Should not be necessary, but oc_physics::step must take a reference ...
    pub nature: NatureIndex,
    pub z: u8,
    // Copy it from nature for performance consideration.
    // If it use too much RAM, consider read it through Mod
    pub allow: Traversability,
}

impl Tile {
    // Apply changes on this tile according to given AlteredTile
    pub fn alter(&mut self, altered: &AlteredTile, mod_: &Mod) {
        self.nature = altered.nature;
        self.allow = mod_.nature(altered.nature).traversability.clone();
    }

    pub fn z_pixels(&self, w: &WorldConfig) -> f32 {
        self.z as f32 * w.geo_meters_per_z().0 * w.geo_pixels_per_meters()
    }
}

pub trait AsTiles {
    fn as_tiles<'a>(&self, world: &'a World) -> Vec<(WorldTileIndex, &'a Tile)>;
}

impl AsTiles for WorldRegionIndex {
    fn as_tiles<'a>(&self, world: &'a World) -> Vec<(WorldTileIndex, &'a Tile)> {
        world.region_tiles(*self)
    }
}

pub trait IntoTiles {
    fn to_tiles(&self, world: &World) -> Vec<(WorldTileIndex, Tile)>;
}

impl IntoTiles for WorldRegionIndex {
    fn to_tiles(&self, world: &World) -> Vec<(WorldTileIndex, Tile)> {
        let tiles = self.as_tiles(world);
        tiles.into_iter().map(|(i, t)| (i, t.clone())).collect()
    }
}

impl Material for Tile {
    fn kind(&self) -> Option<oc_root::material::MaterialKind> {
        None
    }
}

impl Physic for Tile {
    fn position(&self, w: &WorldConfig) -> WorldVec3 {
        let xy: TileXy = self.i.into_(w);
        let point = xy.point(w);
        tracing::trace!(name="DEBUG", i=?self.i, xy=?xy, point=?point);
        WorldVec3::new(
            point[0],
            point[1],
            self.z as f32 * w.geo_meters_per_z().0 * w.geo_pixels_per_meters(),
        )
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        static EMPTY: Vec<Force> = vec![];
        &EMPTY
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        include_underground: bool,
        w: &WorldConfig,
        mod_: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        tracing::trace!(name = "tile-volume", ref_ = ?ref_);
        let nature = mod_.nature(self.nature);
        let exceedance = nature.z.0 * w.geo_pixels_per_meters();

        // Tile "ground". Always not traversable.
        let ground = (
            Volume::Cube {
                x: ref_.x,
                y: ref_.y,
                z: -DEPTH,
                width: w.geo_pixels_per_tile() as f32,
                height: w.geo_pixels_per_tile() as f32,
                depth: DEPTH + ref_.z,
            },
            Traversability::none(),
            Direction::NORTH,
        );
        // Tile nature (hedge part for example)
        let nature = (
            Volume::Cube {
                x: ref_.x,
                y: ref_.y,
                z: ref_.z,
                width: w.geo_pixels_per_tile() as f32,
                height: w.geo_pixels_per_tile() as f32,
                depth: exceedance,
            },
            // TODO: perf test with reference ?
            nature.traversability.clone(),
            Direction::NORTH,
        );

        match include_underground {
            true => vec![ground, nature],
            false => vec![nature],
        }
    }

    fn intercept(&self, kind: MaterialKind, mod_: &Mod) -> f32 {
        let nature = mod_.nature(self.nature);
        match kind {
            MaterialKind::Projectile => nature.intercept.projectile.probability(),
            MaterialKind::Individual => 0.0,
            MaterialKind::Vehicle => 0.0,
        }
    }

    fn ignore_side(&self) -> IgnoreSide {
        IgnoreSide::None
    }

    fn collision_ignorable(&self) -> bool {
        true
    }

    fn side(&self) -> Option<oc_root::side::Side> {
        None
    }

    fn apply_z(&self, w: &WorldConfig) -> Option<f32> {
        Some(self.z_pixels(w))
    }
}

impl Walls for Vec<Tile> {
    fn as_walls(&self, mod_: &Mod) -> Vec<bool> {
        self.iter()
            .map(|tile| {
                // FIXME: When vehicle, will need same but for vehicle
                mod_.nature(tile.nature)
                    .traversability
                    .deny(MaterialKind::Individual)
            })
            .collect::<Vec<bool>>()
    }
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct AlteredTile {
    pub i: WorldTileIndex, // Should not be necessary, but oc_physics::step must take a reference ...
    pub nature: NatureIndex,
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use oc_root::{WcfgFrom, physics::MetersSeconds, side::Side, utils::Frequency};
    use oc_utils::index::AtIndex;

    struct MyObject(WorldVec3, Vec<Force>);
    #[derive(Debug, Clone, Copy, serde::Serialize, PartialEq)]
    struct MyObjectId(usize);

    impl Physic for MyObject {
        fn position(&self, _: &WorldConfig) -> WorldVec3 {
            self.0
        }
        fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
            &self.1
        }

        fn volumes(
            &self,
            ref_: WorldVec3,
            _: bool,
            _: &WorldConfig,
            _: &Mod,
        ) -> Vec<(Volume, Traversability, Direction)> {
            vec![(
                Volume::Point {
                    x: ref_.x,
                    y: ref_.y,
                    z: ref_.z,
                },
                Traversability::all(),
                Direction::NORTH,
            )]
        }

        fn ignore_side(&self) -> IgnoreSide {
            IgnoreSide::None
        }

        fn side(&self) -> Option<Side> {
            None
        }
    }

    impl Material for MyObject {
        fn kind(&self) -> Option<MaterialKind> {
            Some(MaterialKind::Projectile)
        }
    }
    fn workspace_root() -> PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn test_collision_on_tile_zero_z() {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing::level_filters::LevelFilter::TRACE.into())
                    .from_env()
                    .unwrap(),
            )
            .init();

        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(10, 10)
            .with_physics_tick(Frequency::new(1.0))
            .with_geo_pixels_per_meters(10.)
            .with_geo_pixels_per_tile(5);
        let from = glam::Vec3::new(0., 0., 10.);
        let to = glam::Vec3::new(10., 10., 0.);
        let direction = (to - from).normalize_or_zero();
        let direction = WorldVec3::new(direction.x, direction.y, direction.z);

        let speed = MetersSeconds(1000.0);
        let force = Force::Translation(direction, speed);
        // Use `Tile` because we want test volumes
        let tiles: Vec<Tile> = (0..(10 * 10))
            .map(|i| Tile::new(WorldTileIndex(i), NatureIndex(0), 0, Traversability::all()))
            .collect();
        let object = MyObject([0.0, 0.0, 10.0].into(), vec![force]);
        let objects = |xy| -> Vec<(MyObjectId, &dyn Physic)> {
            let tile = WorldTileIndex::from_(TileXy(xy), &w);
            vec![(MyObjectId(1), tiles.at(tile) as &dyn Physic)]
        };

        // When
        let (position, forces, events): (
            WorldVec3,
            Vec<Force>,
            Vec<oc_physics::Event<MyObjectId>>,
        ) = oc_physics::step(
            &w,
            w.physics_tick().period(),
            &mod_,
            (MyObjectId(0), &object),
            objects,
            |_| vec![],
            0,
            "test",
        );

        // Then
        let expected_position: WorldVec3 = [10.0, 10.0, 0.0].into();
        assert_eq!(position, expected_position); // x axis hit first, at MySubject (x) 2.0 (position) + 2.0 (width)
        assert_eq!(forces, Vec::<Force>::new());
        assert_eq!(
            events,
            vec![oc_physics::Event::Collision(
                MyObjectId(0),
                MyObjectId(1),
                Default::default()
            )]
        );
    }
}
