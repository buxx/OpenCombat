#[cfg(test)]
mod test {
    use oc_geo::tile::WorldTileIndex;
    use oc_mod::nature::Traversability;
    use oc_mod::{Mod, nature::NatureIndex};
    use oc_physics::{Event, Force, Physic, collision::Material, volume::Volume};
    #[cfg(test)]
    use oc_root::geo::{WorldVec2, WorldVec3};
    use oc_root::{
        WorldConfig,
        physics::{Meters, MetersSeconds},
    };
    use oc_utils::d2::{Direction, Xy};
    use oc_world::tile::Tile;
    use rstest::rstest;
    use serde::Serialize;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    enum ObjectsId {
        Tile(WorldTileIndex),
        Object(ObjectId),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
    struct ObjectId(usize);
    struct Object(WorldVec3, Vec<Force>);

    impl Physic for Object {
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

        fn ignore_side(&self) -> oc_physics::IgnoreSide {
            oc_physics::IgnoreSide::None
        }

        fn side(&self) -> Option<oc_root::side::Side> {
            None
        }
    }

    impl Material for Object {
        fn kind(&self) -> Option<oc_root::material::MaterialKind> {
            Some(oc_root::material::MaterialKind::Projectile)
        }
    }

    #[rstest]
    // Case 1
    #[case(
        // Object at identical pos than tile, but without force
        (0., 0., Meters(0.)), vec![],
        // produce nothing
        Meters(0.),
        ([0., 0., 0.].into(), vec![], vec![])
    )]
    // Case 2
    #[case(
        // Object at other pos than tile, with movement to the ground, in the tile
        (5.1, 5.1, Meters(0.)), vec![Force::Translation([-1., -1., -1.].into(), MetersSeconds(1.))],
        // produce collision
        Meters(0.),
        ([4.0, 4.0, -2.0].into(), vec![], vec![Event::Collision(ObjectsId::Object(ObjectId(0)), ObjectsId::Tile(WorldTileIndex(0)), Default::default())])
    )]
    // Case 3
    #[case(
        // Incoming object at 10 meters
        (5.1, 5.1, Meters(10.)), vec![Force::Translation([-1., -1., 0.].into(), MetersSeconds(1.))],
        // produce collision with a tile at 12 meters
        Meters(12.),
        ([4.0, 4.0, 50.0].into(), vec![], vec![Event::Collision(ObjectsId::Object(ObjectId(0)), ObjectsId::Tile(WorldTileIndex(0)), Default::default())])
    )]
    fn test_tile_collision_in_meters_zero(
        #[case] object_pos: (f32, f32, Meters),
        #[case] object_forces: Vec<Force>,
        #[case] tile_meters: Meters,
        #[case] expected: (WorldVec3, Vec<Force>, Vec<Event<ObjectsId>>),
    ) {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let geo_meters_per_z = Meters(0.1);
        let w = WorldConfig::new(1000, 1000)
            .with_geo_pixels_per_tile(5)
            .with_geo_pixels_per_meters(5.)
            .with_geo_meters_per_z(geo_meters_per_z);

        let tile_i = WorldTileIndex(0);
        let tile_z = (tile_meters.0 / geo_meters_per_z.0) as u8;
        let tile = Tile {
            i: tile_i,
            nature: NatureIndex(0),
            z: tile_z,
            allow: Traversability::all(),
        };

        let object_i = ObjectId(0);
        let object_x = object_pos.0;
        let object_y = object_pos.1;
        let object_z = object_pos.2.0 * w.geo_pixels_per_meters();
        let object = Object(WorldVec3::new(object_x, object_y, object_z), object_forces);
        let tile: &dyn Physic = &tile;

        let objects = |xy| {
            if xy == Xy(0, 0) {
                vec![(ObjectsId::Tile(tile_i), tile)]
            } else {
                vec![]
            }
        };

        // When
        let delta = 1.0;
        let result = oc_physics::step(
            &w,
            delta,
            &mod_,
            (ObjectsId::Object(object_i), &object),
            objects,
            |_| vec![],
            0,
            "tests",
        );

        assert_eq!(result, expected);
    }

    struct StopObject(Object, WorldVec3);

    impl Material for StopObject {
        fn kind(&self) -> Option<oc_root::material::MaterialKind> {
            self.0.kind()
        }
    }

    impl Physic for StopObject {
        fn position(&self, w: &WorldConfig) -> WorldVec3 {
            self.0.position(w)
        }

        fn forces(&self, w: &WorldConfig) -> &Vec<Force> {
            self.0.forces(w)
        }

        fn volumes(
            &self,
            ref_: WorldVec3,
            underground: bool,
            w: &WorldConfig,
            mod_: &Mod,
        ) -> Vec<(Volume, Traversability, Direction)> {
            self.0.volumes(ref_, underground, w, mod_)
        }

        fn ignore_side(&self) -> oc_physics::IgnoreSide {
            self.0.ignore_side()
        }

        fn side(&self) -> Option<oc_root::side::Side> {
            self.0.side()
        }

        fn want_stop(&self) -> Option<(WorldVec2, f32)> {
            Some((WorldVec2::new(self.1.x, self.1.y), 2.))
        }
    }

    #[rstest]
    #[case(
        [120., 100., 0.],
        ([118., 100., 0.].into(), vec![], vec![Event::WantStop(ObjectsId::Object(ObjectId(0)), [118., 100., 0.].into())])
    )]
    // Stop point near the way, but no line pixel near enough: stop on it (no overshoot)
    #[case(
        [130.5, 102.5, 0.],
        ([129., 103., 0.].into(), vec![], vec![Event::WantStop(ObjectsId::Object(ObjectId(0)), [129., 103., 0.].into())])
    )]
    // Stop point is not on the way: no stop, force kept
    #[case(
        [120., 200., 0.],
        (
            [150., 100., 0.].into(),
            vec![Force::Translation([1., 0., 0.].into(), MetersSeconds(10.))],
            vec![]
        )
    )]
    fn test_want_stop(
        #[case] stop_at: [f32; 3],
        #[case] expected: (WorldVec3, Vec<Force>, Vec<Event<ObjectsId>>),
    ) {
        // Given
        let mod_ = Mod::load(&workspace_root().join("mods/tests1"), None).unwrap();
        let w = WorldConfig::new(1000, 1000)
            .with_geo_pixels_per_tile(5)
            .with_geo_pixels_per_meters(5.)
            .with_geo_meters_per_z(Meters(0.1));
        // 10 m/s during 1 second at 5 pixels/meter: 50 pixels along x
        let forces = vec![Force::Translation([1., 0., 0.].into(), MetersSeconds(10.))];
        let object = StopObject(
            Object(WorldVec3::new(100., 100., 0.), forces),
            stop_at.into(),
        );

        // When
        let result = oc_physics::step(
            &w,
            1.0,
            &mod_,
            (ObjectsId::Object(ObjectId(0)), &object),
            |_| vec![],
            |_| vec![],
            0,
            "tests",
        );

        // Then
        assert_eq!(result, expected);
    }
}
