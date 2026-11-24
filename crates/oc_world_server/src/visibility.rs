use derive_more::Constructor;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{BodyGesture, IndividualIndex, behavior::Behavior};
use oc_lov::Modifier;
use oc_root::side::{BySide, Side};
use oc_root::{WcfgFrom, WorldConfig, opacity::CumulatedOpacity};
use oc_utils::index::AtIndex;
use oc_utils::{d2::Xy, polygon::Polygon};
use oc_vehicle::VehicleIndex;
use oc_world::{World, visibility::Visibility};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::index::Indexes;

#[derive(Constructor)]
pub struct Processor<'a> {
    world: &'a World,
    index: &'a Indexes,
}

impl<'a> Processor<'a> {
    /// Visibilities of all opposite sides pairs
    #[allow(dead_code)]
    pub fn compute(&self) -> Vec<(IndividualIndex, IndividualIndex, Visibility)> {
        let a = self.index.side_individuals(Side::A);
        let b = self.index.side_individuals(Side::B);
        let mut visibilities = self.compute_individuals_for(a, b);
        visibilities.extend(self.compute_individuals_for(b, a));
        visibilities
    }

    /// For each world interior, tells if one of given individuals is inside it
    fn occupied_interiors(&self, individuals: &[IndividualIndex]) -> Vec<bool> {
        self.world
            .interiors
            .iter()
            .map(|interior| {
                individuals.iter().any(|i| {
                    let individual = self.world.individual(*i);
                    individual.status.can_step() && interior.contains(individual.position.into())
                })
            })
            .collect()
    }

    /// Visibilities of given watchers on given targets. Computing by chunks of watchers, with
    /// world locks released between chunks, avoids to block writers during the whole computation.
    pub fn compute_individuals_for(
        &self,
        watchers: &[IndividualIndex],
        targets: &[IndividualIndex],
    ) -> Vec<(IndividualIndex, IndividualIndex, Visibility)> {
        let w = &self.world.w;
        let mod_ = &self.world.mod_;
        let occupied =
            BySide::from_fn(|side| self.occupied_interiors(self.index.side_individuals(side)));

        let compute_pair = |i1: &IndividualIndex, i2: &IndividualIndex| {
            tracing::trace!(name = "visibility-processor-compute", i1=?i1, i2=?i2);

            let individual1 = self.world.individual(*i1);
            if !individual1.can_lov() {
                tracing::trace!(name="visibility-processor-compute-cant-lov", i1=?i1, i2=?i2);
                return None;
            }

            let individual2 = self.world.individual(*i2);
            if !individual2.is_lov_target() {
                tracing::trace!(name="visibility-processor-compute-cant-be-lov", i1=?i1, i2=?i2);
                return None;
            }

            // Individuals inside an interior are hidden from opposite having nobody inside it,
            // except when they stand on an altered tile.
            let on_altered_tile = self.world.altered_tiles.at(individual2.tile).is_some();
            let occupied = &occupied[individual1.side];
            let hidden_by_interior = !on_altered_tile
                && self
                    .world
                    .interiors
                    .iter()
                    .zip(occupied)
                    .any(|(interior, occupied)| {
                        !occupied && interior.contains(individual2.position.into())
                    });
            if hidden_by_interior {
                tracing::trace!(name="visibility-processor-compute-in-interior", i1=?i1, i2=?i2);
                return Some((*i1, *i2, Visibility::default()));
            }

            let p1 = individual1
                .position
                .add_z(individual1.gesture.body.weapon_z().pixels(w));
            let p2 = individual2
                .position
                .add_z(individual2.gesture.body.target_z().pixels(w));

            let is_crawling = matches!(
                individual2.gesture.body,
                BodyGesture::Crawling(_) | BodyGesture::Prone(_)
            );
            let is_hiding = matches!(individual2.behavior, Behavior::Hide(_));

            let modifier = match (is_hiding, is_crawling) {
                (true, _) => Modifier::Hiding,
                (_, true) => Modifier::Crawling,
                _ => Modifier::None(true),
            };

            // Watcher and target own vehicles (when embarked) don't block the view
            let ignore: Vec<VehicleIndex> = [individual1.embarked, individual2.embarked]
                .into_iter()
                .flatten()
                .map(|(vehicle, _)| vehicle)
                .collect();
            let at = |xy, z| path_objects_at(w, mod_, self.world, self.index, &ignore, xy, z);
            let visibility = visibility(w, at, p1, p2, modifier);
            Some((*i1, *i2, visibility))
        };

        // Pairs are independent: compute them in parallel (one task by watcher), keeping order
        watchers
            .par_iter()
            .flat_map_iter(|i1| targets.iter().filter_map(|i2| compute_pair(i1, i2)))
            .collect::<Vec<_>>()
    }

    /// Visibilities of given watchers on opposite side vehicles. A vehicle is visible when one of
    /// its chassis sight points is.
    pub fn compute_vehicles_for(
        &self,
        watchers: &[IndividualIndex],
    ) -> Vec<(IndividualIndex, VehicleIndex, Visibility)> {
        let vehicles = self.vehicles_sights();
        if vehicles.is_empty() {
            return vec![];
        }

        // Watchers are independent: compute them in parallel, keeping order
        watchers
            .par_iter()
            .flat_map_iter(|i1| self.watcher_vehicles_visibilities(*i1, &vehicles))
            .collect::<Vec<_>>()
    }

    /// Chassis sight points of all world vehicles (computed once for all watchers)
    fn vehicles_sights(&self) -> Vec<VehicleSight> {
        let (w, mod_) = (&self.world.w, &self.world.mod_);
        self.world
            .vehicles()
            .iter()
            .enumerate()
            .map(|(i, vehicle)| VehicleSight {
                i: VehicleIndex(i as u64),
                side: vehicle.side,
                points: vehicle.chassis_sight_points(w, mod_),
            })
            .collect()
    }

    /// Visibilities of given watcher on opposite side vehicles
    fn watcher_vehicles_visibilities(
        &self,
        i1: IndividualIndex,
        vehicles: &[VehicleSight],
    ) -> Vec<(IndividualIndex, VehicleIndex, Visibility)> {
        let watcher = self.world.individual(i1);
        if !watcher.can_lov() {
            return vec![];
        }
        let from = watcher
            .position
            .add_z(watcher.gesture.body.weapon_z().pixels(&self.world.w));

        vehicles
            .iter()
            .filter(|vehicle| vehicle.side != watcher.side)
            .map(|vehicle| {
                // Watcher own vehicle (when embarked) and target vehicle don't block the view
                let ignore: Vec<VehicleIndex> = watcher
                    .embarked
                    .map(|(vehicle, _)| vehicle)
                    .into_iter()
                    .chain([vehicle.i])
                    .collect();
                let visibility = self.chassis_visibility(from, &vehicle.points, &ignore);
                tracing::trace!(name = "visibility-processor-compute-vehicle", i1=?i1, i2=?vehicle.i, visible=visibility.visible);
                (i1, vehicle.i, visibility)
            })
            .collect()
    }

    /// Visibility of the most visible chassis point from given position
    fn chassis_visibility(
        &self,
        from: oc_root::geo::WorldVec3,
        points: &[oc_root::geo::WorldVec3],
        ignore: &[VehicleIndex],
    ) -> Visibility {
        let (w, mod_) = (&self.world.w, &self.world.mod_);
        let at = |xy, z| path_objects_at(w, mod_, self.world, self.index, ignore, xy, z);
        points
            .iter()
            .map(|to| visibility(w, at, from, *to, Modifier::None(true)))
            .min_by(|a, b| a.opacity.0.total_cmp(&b.opacity.0))
            .unwrap_or_default()
    }
}

/// Vehicle points to test for its visibility
struct VehicleSight {
    i: VehicleIndex,
    side: Side,
    points: Vec<oc_root::geo::WorldVec3>,
}

pub fn visibility<I: IntoIterator<Item = oc_lov::Step>>(
    w: &WorldConfig,
    at: impl Fn(Xy, f32) -> I,
    p1: oc_root::geo::WorldVec3,
    p2: oc_root::geo::WorldVec3,
    modifier: Modifier,
) -> Visibility {
    let ignore = w.ignore_firsts_lov_tiles() as usize;
    let lov = oc_lov::PathBuilder::new(w, at).build(p1, p2, ignore, modifier);

    let opacity = lov
        .sections
        .last()
        .map(|s| s.opacity)
        .unwrap_or(CumulatedOpacity(1.0));
    let visible = opacity <= w.individual_visibility_until();

    tracing::trace!(
        name="visibility-processor-compute-result",
        visible=visible,
        opacity=?opacity
    );

    #[cfg(not(feature = "debug"))]
    return Visibility::new(visible, opacity);

    #[cfg(feature = "debug")]
    return Visibility::new(
        visible,
        opacity,
        lov.sections
            .clone()
            .into_iter()
            .map(|s| (s.start, s.stop, s.opacity))
            .collect::<Vec<_>>(),
    );
}

/// Lov steps at given position. Vehicles chassis block the view, except `ignore` ones (like
/// vehicle of the watcher crew, or the target vehicle itself).
pub fn path_objects_at(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    world: &World,
    indexes: &Indexes,
    ignore: &[VehicleIndex],
    at: Xy,
    z: f32,
) -> Vec<oc_lov::Step> {
    let i = WorldTileIndex::from_(TileXy(at), w);
    let smoke = world.smoke_at(i);
    world
        .tile(i)
        .map(|t| {
            let tile_z = t.z as f32 * w.geo_meters_per_z().0 * w.geo_pixels_per_meters();
            let relative_z = z - tile_z;
            let nature = mod_.nature(t.nature);
            let mut steps = vec![oc_lov::Step::from_nature(w, nature, relative_z)];

            // Smoke
            if !smoke.is_zero() {
                steps.push(oc_lov::Step::from_smoke(smoke));
            }

            // Wreck chassis blocks the view
            if indexes
                .tile_wrecks(i)
                .iter()
                .any(|wreck| world.wreck(*wreck).occupies_z(z, w, mod_))
            {
                steps.push(oc_lov::Step::opaque());
            }

            // Vehicle chassis blocks the view
            if indexes.tile_vehicles(i).iter().any(|vehicle| {
                !ignore.contains(vehicle) && world.vehicle(*vehicle).occupies_z(z, w, mod_)
            }) {
                steps.push(oc_lov::Step::opaque());
            }

            tracing::trace!(name="visibility-path-objects-at", at=?at, z=z, t=?t, tile_z=?tile_z, relative_z=?relative_z, steps=?steps);
            steps
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use ::tests::{individual::TestIndividual, world::TestWorld};
    use oc_root::{
        geo::{WorldVec2, WorldVec3},
        side::Side,
    };
    use oc_world::interior::Interior;
    use rstest::rstest;

    use super::*;

    const WATCHER: IndividualIndex = IndividualIndex(0);
    const TARGET: IndividualIndex = IndividualIndex(1);

    fn interior_around(center: WorldVec2) -> Interior {
        let points = vec![
            WorldVec2::new(center.x - 10., center.y - 10.),
            WorldVec2::new(center.x + 10., center.y - 10.),
            WorldVec2::new(center.x + 10., center.y + 10.),
            WorldVec2::new(center.x - 10., center.y + 10.),
        ];
        Interior::new(0, points[0], points, 0., 0., None)
    }

    /// Side A watcher at (10, 10) and side B target at (50, 50), plus given extra individuals.
    /// Returns visibility of the target by the watcher.
    fn target_visible(
        interiors: Vec<Interior>,
        extra: Vec<(Side, WorldVec3)>,
        watcher_position: WorldVec3,
    ) -> bool {
        target_visible_(interiors, extra, watcher_position, false)
    }

    fn target_visible_(
        interiors: Vec<Interior>,
        extra: Vec<(Side, WorldVec3)>,
        watcher_position: WorldVec3,
        target_on_altered_tile: bool,
    ) -> bool {
        let w = WorldConfig::new(100, 100);
        let mut individuals = vec![
            TestIndividual::builder()
                .side(Side::A)
                .position(watcher_position)
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(Side::B)
                .position(WorldVec3::new(50., 50., 0.))
                .build()
                .make(&w),
        ];
        for (side, position) in extra {
            individuals.push(
                TestIndividual::builder()
                    .side(side)
                    .position(position)
                    .build()
                    .make(&w),
            );
        }
        let mut world = TestWorld::builder()
            .individuals(individuals)
            .interiors(interiors)
            .build()
            .make(&w);
        if target_on_altered_tile {
            let tile = world.individual(TARGET).tile;
            let nature = world.tile(tile).expect("Tile must exist").nature;
            *world.altered_tiles.at_mut(tile) =
                Some(oc_world::tile::AlteredTile::new(tile, nature));
        }
        let indexes = Indexes::new(&world, &w);

        let visibilities =
            Processor::new(&world, &indexes).compute_individuals_for(&[WATCHER], &[TARGET]);

        let (_, _, visibility) = visibilities
            .into_iter()
            .find(|(i1, i2, _)| *i1 == WATCHER && *i2 == TARGET)
            .expect("Pair must be computed");
        visibility.visible
    }

    #[test]
    fn test_smoke_between_watcher_and_target_hides_target() {
        let w = WorldConfig::new(100, 100);
        let individuals = vec![
            TestIndividual::builder()
                .side(Side::A)
                .position(WorldVec3::new(10., 10., 0.))
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(Side::B)
                .position(WorldVec3::new(50., 50., 0.))
                .build()
                .make(&w),
        ];
        let mut world = TestWorld::builder()
            .individuals(individuals)
            .build()
            .make(&w);
        // Dense smoke on every tile
        world
            .smoke
            .iter_mut()
            .for_each(|s| *s = oc_utils::percent::Percent::FULL);
        let indexes = Indexes::new(&world, &w);

        let visibilities =
            Processor::new(&world, &indexes).compute_individuals_for(&[WATCHER], &[TARGET]);

        let (_, _, visibility) = visibilities.into_iter().next().expect("Pair computed");
        assert!(!visibility.visible);
    }

    #[test]
    fn test_wreck_between_watcher_and_target_hides_target() {
        let w = WorldConfig::new(100, 100);
        let individuals = vec![
            TestIndividual::builder()
                .side(Side::A)
                .position(WorldVec3::new(10., 10., 0.))
                .build()
                .make(&w),
            TestIndividual::builder()
                .side(Side::B)
                .position(WorldVec3::new(50., 50., 0.))
                .build()
                .make(&w),
        ];
        // Wreck chassis across the line of view
        let wreck = oc_world::wreck::Wreck::Vehicle {
            type_: oc_mod::vehicle::VehicleIndex(0),
            position: WorldVec3::new(30., 30., 0.),
            chassis_direction: oc_utils::d2::Direction::new(1., -1.),
        };
        let world = TestWorld::builder()
            .individuals(individuals)
            .wrecks(vec![wreck])
            .build()
            .make(&w);
        let indexes = Indexes::new(&world, &w);

        let visibilities =
            Processor::new(&world, &indexes).compute_individuals_for(&[WATCHER], &[TARGET]);

        let (_, _, visibility) = visibilities.into_iter().next().expect("Pair computed");
        assert!(!visibility.visible);
    }

    /// Side A watcher at `watcher` (embarked in the vehicle if `embarked`), side B target at
    /// (50, 50) and a side A vehicle chassis at `vehicle`. Returns visibility of the target.
    fn target_visible_with_vehicle(watcher: WorldVec3, embarked: bool, vehicle: WorldVec3) -> bool {
        let w = WorldConfig::new(100, 100);
        let embarked = embarked.then_some((VehicleIndex(0), oc_mod::vehicle::CrewRole::Commander));
        let individuals = vec![
            TestIndividual::builder()
                .side(Side::A)
                .position(watcher)
                .build()
                .make(&w)
                .with_embarked(embarked),
            TestIndividual::builder()
                .side(Side::B)
                .position(WorldVec3::new(50., 50., 0.))
                .build()
                .make(&w),
        ];
        let vehicle = oc_vehicle::Vehicle::new(
            Side::A,
            oc_mod::vehicle::VehicleIndex(0),
            oc_mod::illustration::IllustrationIndex(0),
            vehicle,
            vec![],
            oc_utils::d2::Direction::new(1., -1.),
            None,
        );
        let world = TestWorld::builder()
            .individuals(individuals)
            .vehicles(vec![vehicle])
            .build()
            .make(&w);
        let indexes = Indexes::new(&world, &w);

        let visibilities =
            Processor::new(&world, &indexes).compute_individuals_for(&[WATCHER], &[TARGET]);

        let (_, _, visibility) = visibilities.into_iter().next().expect("Pair computed");
        visibility.visible
    }

    #[test]
    fn test_vehicle_between_watcher_and_target_hides_target() {
        let watcher = WorldVec3::new(10., 10., 0.);
        assert!(!target_visible_with_vehicle(
            watcher,
            false,
            WorldVec3::new(30., 30., 0.)
        ));
    }

    #[test]
    fn test_vehicle_aside_watcher_and_target_does_not_hide_target() {
        let watcher = WorldVec3::new(10., 10., 0.);
        assert!(target_visible_with_vehicle(
            watcher,
            false,
            WorldVec3::new(80., 10., 0.)
        ));
    }

    #[test]
    fn test_embarked_watcher_own_vehicle_does_not_hide_target() {
        let watcher = WorldVec3::new(30., 30., 0.);
        assert!(target_visible_with_vehicle(watcher, true, watcher));
    }

    #[test]
    fn test_target_visible_without_interior() {
        assert!(target_visible(vec![], vec![], WorldVec3::new(10., 10., 0.)));
    }

    #[test]
    fn test_target_in_interior_without_watcher_side_inside_is_not_visible() {
        let interior = interior_around(WorldVec2::new(50., 50.));

        assert!(!target_visible(
            vec![interior],
            vec![],
            WorldVec3::new(10., 10., 0.)
        ));
    }

    #[test]
    fn test_target_in_interior_on_altered_tile_is_visible() {
        let interior = interior_around(WorldVec2::new(50., 50.));

        assert!(target_visible_(
            vec![interior],
            vec![],
            WorldVec3::new(10., 10., 0.),
            true
        ));
    }

    #[test]
    fn test_target_in_interior_with_watcher_inside_is_visible() {
        let interior = interior_around(WorldVec2::new(50., 50.));

        assert!(target_visible(
            vec![interior],
            vec![],
            WorldVec3::new(45., 45., 0.)
        ));
    }

    #[test]
    fn test_target_in_interior_with_watcher_side_ally_inside_is_visible() {
        let interior = interior_around(WorldVec2::new(50., 50.));

        assert!(target_visible(
            vec![interior],
            vec![(Side::A, WorldVec3::new(55., 55., 0.))],
            WorldVec3::new(10., 10., 0.)
        ));
    }

    #[test]
    fn test_target_in_interior_with_only_enemy_side_inside_is_not_visible() {
        let interior = interior_around(WorldVec2::new(50., 50.));

        // Target side B is itself inside, but another B individual changes nothing for side A
        assert!(!target_visible(
            vec![interior],
            vec![(Side::B, WorldVec3::new(55., 55., 0.))],
            WorldVec3::new(10., 10., 0.)
        ));
    }

    #[rstest]
    #[case(WorldVec2::new(10., 50.))]
    #[case(WorldVec2::new(90., 90.))]
    fn test_target_outside_interior_is_visible(#[case] center: WorldVec2) {
        let interior = interior_around(center);

        assert!(target_visible(
            vec![interior],
            vec![],
            WorldVec3::new(10., 10., 0.)
        ));
    }
}
