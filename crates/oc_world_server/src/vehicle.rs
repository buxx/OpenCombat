use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::IndividualIndex;
use oc_mod::Mod;
use oc_network::ToClient;
use oc_physics::Physic;
use oc_root::{WcfgInto, WorldConfig, geo::WorldVec2, physics::Meters};
use oc_utils::{
    d2::{Direction, Xy},
    index::AtIndex,
};
use oc_vehicle::{Update, VehicleIndex, network};
use oc_world::{World, utils::oriented_rectangle_covered_tiles};

use crate::{
    index::{self, Indexes, vehicle_region},
    routing::Listening,
    runner,
    state::ObjectId,
};

pub fn write(
    world: &mut World,
    update: Update,
    i: VehicleIndex,
) -> (Vec<(Listening, Vec<ToClient>)>, Vec<index::Effect>) {
    let w = &world.w;
    let size = world.mod_.vehicle(world.vehicles.at(i).type_).size;
    let vehicle = world.vehicles.at_mut(i);
    let before = (WorldVec2::from(vehicle.position), vehicle.chassis_direction);

    vehicle.apply(&update);

    let now = (WorldVec2::from(vehicle.position), vehicle.chassis_direction);
    let (region_before, region) = (vehicle_region(before.0, w), vehicle_region(now.0, w));

    let update = ToClient::Vehicle(network::Vehicle::Update(i, update));
    let mut outputs = vec![(Listening::Regions(vec![region]), vec![update])];

    if region_before != region {
        let insert = ToClient::Vehicle(network::Vehicle::Insert(i, vehicle.clone()));
        let forgot = ToClient::Vehicle(network::Vehicle::Forgot(i));
        outputs.push((Listening::EnterBorder(region_before, region), vec![insert]));
        outputs.push((Listening::ExitBorder(region_before, region), vec![forgot]));
    }

    let mut effects = vec![];
    if before != now {
        let effect = index::VehicleEffect::Placement { size, now, before };
        effects.push(index::Effect::Vehicle(i, effect));
    }

    (outputs, effects)
}

/// Physics of `vehicles` during `delta` seconds: chassis and turret rotations toward their targets,
/// then forces application (crew follows its vehicle).
pub fn step(
    world: &World,
    indexes: &Indexes,
    vehicles: &[(VehicleIndex, &oc_vehicle::Vehicle)],
    w: &WorldConfig,
    mod_: &Mod,
    delta: f32,
) -> Vec<runner::update::Update> {
    let mut updates = vec![];

    for (i, vehicle) in vehicles {
        let (i, vehicle) = (*i, *vehicle);
        let type_ = mod_.vehicle(vehicle.type_);
        let mut push = |update| updates.push(runner::update::Update::UpdateVehicle(i, update));

        let (chassis, turret) = vehicle.rotate(type_, delta);
        if chassis != vehicle.chassis_direction {
            push(Update::SetChassisOrientation(chassis));
        }
        if let Some(turret) = turret
            && Some(turret) != vehicle.turret_direction
        {
            push(Update::SetTurretDirection(turret));
        }

        if vehicle.forces.is_empty() {
            continue;
        }

        // Vehicles collide with world tiles and wrecks
        let size = type_.size;
        let orientation = vehicle.chassis_direction;
        let collision_objects =
            |xy: Xy| chassis_tiles_objects(world, indexes, w, xy, size, orientation);
        let (position, forces, _) = oc_physics::step(
            w,
            delta,
            mod_,
            (i, vehicle),
            collision_objects,
            |_| vec![],
            0,
            "server-vehicle",
        );
        tracing::trace!(name="physics-vehicle", i=?i, position=?position, forces=?forces);

        if position != vehicle.position {
            push(Update::SetPosition(position));
            for member in crew(world, i) {
                updates.push(runner::update::Update::UpdateIndividual(
                    member,
                    oc_individual::Update::SetPosition(position),
                ));
            }
        }
        if forces != vehicle.forces {
            updates.push(runner::update::Update::UpdateVehicle(
                i,
                Update::SetForces(forces),
            ));
        }
    }

    updates
}

/// Tiles under the chassis of a vehicle.
fn chassis_tiles_objects<'a>(
    world: &'a World,
    indexes: &Indexes,
    w: &WorldConfig,
    xy: Xy,                 // Will consider center of this tile
    size: (Meters, Meters), // size of the chassis
    orientation: Direction, // Orientation of the chassis
) -> Vec<(ObjectId, &'a dyn Physic)> {
    let center: WorldTileIndex = TileXy(xy).into_(w);
    let tile = w.geo_pixels_per_tile() as f32;
    let center_ = [(xy.0 as f32 + 0.5) * tile, (xy.1 as f32 + 0.5) * tile];
    // Vehicle center is somewhere on the tile `xy`: enlarge by one tile to cover all its positions
    let (width, length) = (size.0.pixels(w) + tile, size.1.pixels(w) + tile);

    let tiles: Vec<WorldTileIndex> =
        oriented_rectangle_covered_tiles(center_, width, length, orientation, w)
            .into_iter()
            .filter(|i| *i != center)
            .chain(std::iter::once(center))
            .collect();

    // Wrecks covering these tiles (each one once)
    let mut wrecks: Vec<(ObjectId, &dyn Physic)> = vec![];
    for i in tiles.iter().flat_map(|i| indexes.tile_wrecks(*i)) {
        if !wrecks.iter().any(|(id, _)| *id == ObjectId::Wreck(*i)) {
            wrecks.push((ObjectId::Wreck(*i), world.wreck(*i) as &dyn Physic));
        }
    }

    tiles
        .into_iter()
        .filter_map(|i| world.tile(i))
        .map(|tile| (ObjectId::Tile(tile.i), tile as &dyn Physic))
        .chain(wrecks)
        .collect()
}

pub fn crew(world: &World, i: VehicleIndex) -> Vec<IndividualIndex> {
    world
        .individuals()
        .iter()
        .enumerate()
        .filter(|(_, individual)| individual.embarked.is_some_and(|(i_, _)| i_ == i))
        .map(|(i, _)| IndividualIndex(i as u64))
        .collect()
}

impl From<VehicleIndex> for ObjectId {
    fn from(value: VehicleIndex) -> Self {
        ObjectId::Vehicle(value)
    }
}

#[cfg(test)]
mod tests {
    use ::tests::{individual::TestIndividual, world::TestWorld};
    use oc_mod::{illustration::IllustrationIndex, vehicle::CrewRole};
    use oc_physics::Force;
    use oc_root::{geo::WorldVec3, physics::MetersSeconds, side::Side};
    use oc_utils::d2::Direction;

    use oc_utils::collections::WithIds;

    use super::*;

    fn world(vehicle: impl FnOnce(&Mod) -> oc_vehicle::Vehicle) -> World {
        let w = WorldConfig::new(100, 100);
        let mod_ = TestWorld::builder().build().make(&w).mod_;
        let vehicle = vehicle(&mod_);
        let crew = TestIndividual::builder()
            .position(vehicle.position)
            .build()
            .make(&w)
            .with_embarked(Some((VehicleIndex(0), CrewRole::Driver)));
        let other = TestIndividual::builder()
            .position(vehicle.position)
            .build()
            .make(&w);
        TestWorld::builder()
            .individuals(vec![crew, other])
            .vehicles(vec![vehicle])
            .build()
            .make(&w)
    }

    fn tank(mod_: &Mod) -> oc_vehicle::Vehicle {
        oc_vehicle::Vehicle::new(
            Side::A,
            mod_.vehicle_from_name("Tank1").unwrap().index(),
            IllustrationIndex(0),
            WorldVec3::new(250., 250., 0.),
            vec![],
            Direction::SOUTH,
            Some(Direction::SOUTH),
        )
    }

    #[test]
    fn vehicle_translates_with_its_crew() {
        // Given: tank moving south at 5m/s (5 pixels per meter)
        let force = Force::Translation(WorldVec3::new(0., 1., 0.), MetersSeconds(5.));
        let world = world(|mod_| {
            let mut tank = tank(mod_);
            tank.forces = vec![force.clone()];
            tank
        });

        // When: one second
        let updates = step(
            &world,
            &Indexes::new(&world, &world.w),
            &world.vehicles().with_ids(),
            &world.w,
            &world.mod_,
            1.0,
        );

        // Then: vehicle and its crew (only) moved
        let position = WorldVec3::new(250., 275., 0.);
        assert_eq!(
            updates,
            vec![
                runner::update::Update::UpdateVehicle(
                    VehicleIndex(0),
                    Update::SetPosition(position)
                ),
                runner::update::Update::UpdateIndividual(
                    IndividualIndex(0),
                    oc_individual::Update::SetPosition(position)
                ),
            ]
        );
    }

    /// Tank (13.5px x 32.5px chassis) at center of tile (50, 50), moving south at 25px/s, with
    /// "BigRock" tiles (blocking) at given tiles
    fn tank_world_with_rocks(rocks: &[(u64, u64)]) -> World {
        let w = WorldConfig::new(100, 100);
        let base = TestWorld::builder().build().make(&w);
        let rock = base.mod_.nature_index("BigRock").unwrap();
        let tiles = base
            .tiles
            .iter()
            .map(|tile| {
                let TileXy(Xy(x, y)) = tile.i.into_(&w);
                match rocks.contains(&(x, y)) {
                    true => oc_world::tile::Tile::new(
                        tile.i,
                        rock,
                        0,
                        base.mod_.nature(rock).traversability.clone(),
                    ),
                    false => tile.clone(),
                }
            })
            .collect();
        let mut tank = tank(&base.mod_);
        tank.position = WorldVec3::new(252.5, 252.5, 0.);
        tank.forces = vec![Force::Translation(
            WorldVec3::new(0., 1., 0.),
            MetersSeconds(5.),
        )];

        TestWorld::builder()
            .tiles(tiles)
            .vehicles(vec![tank])
            .build()
            .make(&w)
    }

    fn moved_to(updates: &[runner::update::Update]) -> Option<WorldVec3> {
        updates.iter().find_map(|u| match u {
            runner::update::Update::UpdateVehicle(_, Update::SetPosition(p)) => Some(*p),
            _ => None,
        })
    }

    #[test]
    fn vehicle_chassis_front_stopped_by_obstacle() {
        // Given: rocks on tiles row y=55 (pixels 275 to 280), in front of the tank
        let world = tank_world_with_rocks(&[(49, 55), (50, 55), (51, 55)]);

        // When: one second (center would reach 277.5 without obstacle)
        let updates = step(
            &world,
            &Indexes::new(&world, &world.w),
            &world.vehicles().with_ids(),
            &world.w,
            &world.mod_,
            1.0,
        );

        // Then: stopped when chassis front (16.25px ahead of center) touch rocks
        let position = moved_to(&updates).unwrap();
        assert!(position.y > 255. && position.y < 260., "{position:?}");
        assert!(updates.contains(&runner::update::Update::UpdateVehicle(
            VehicleIndex(0),
            Update::SetForces(vec![])
        )));
    }

    #[test]
    fn vehicle_stopped_by_wreck() {
        // Given: tank moving south, a wreck (same chassis) 50px ahead
        let w = WorldConfig::new(100, 100);
        let base = tank_world_with_rocks(&[]);
        let tank = base.vehicles[0].clone();
        let wreck = oc_world::wreck::Wreck::Vehicle {
            type_: tank.type_,
            position: WorldVec3::new(252.5, 302.5, 0.),
            chassis_direction: Direction::SOUTH,
        };
        let world = TestWorld::builder()
            .vehicles(vec![tank])
            .wrecks(vec![wreck])
            .build()
            .make(&w);

        // When: one second (center would reach 277.5 without wreck)
        let updates = step(
            &world,
            &Indexes::new(&world, &world.w),
            &world.vehicles().with_ids(),
            &world.w,
            &world.mod_,
            1.0,
        );

        // Then: stopped when chassis front (16.25px ahead of center) touch the wreck back (286.25)
        let position = moved_to(&updates).unwrap_or(WorldVec3::new(252.5, 252.5, 0.));
        assert!(position.y < 272., "{position:?}");
        assert!(updates.contains(&runner::update::Update::UpdateVehicle(
            VehicleIndex(0),
            Update::SetForces(vec![])
        )));
    }

    #[test]
    fn vehicle_chassis_passes_beside_obstacle() {
        // Given: rocks on tile x=53 (pixels 265 to 270), beside chassis (x from 245.75 to 259.25)
        let world = tank_world_with_rocks(&[(53, 52), (53, 55), (53, 57)]);

        // When
        let updates = step(
            &world,
            &Indexes::new(&world, &world.w),
            &world.vehicles().with_ids(),
            &world.w,
            &world.mod_,
            1.0,
        );

        // Then
        let position = moved_to(&updates).unwrap();
        assert!((position.y - 277.5).abs() < 1e-3, "{position:?}");
    }

    #[test]
    fn vehicle_stops_at_destination() {
        // Given
        let destination = WorldVec2::new(250., 260.);
        let world = world(|mod_| {
            let mut tank = tank(mod_);
            tank.forces = vec![Force::Translation(
                WorldVec3::new(0., 1., 0.),
                MetersSeconds(5.),
            )];
            tank.destination = Some(destination);
            tank
        });

        // When
        let updates = step(
            &world,
            &Indexes::new(&world, &world.w),
            &world.vehicles().with_ids(),
            &world.w,
            &world.mod_,
            1.0,
        );

        // Then: stopped at destination (with tolerance), force consumed
        let position = updates.iter().find_map(|u| match u {
            runner::update::Update::UpdateVehicle(_, Update::SetPosition(p)) => Some(*p),
            _ => None,
        });
        let position = WorldVec2::from(position.unwrap());
        assert!(
            glam::Vec2::from(position).distance(destination.into())
                <= oc_vehicle::DESTINATION_TOLERANCE
        );
        assert!(updates.contains(&runner::update::Update::UpdateVehicle(
            VehicleIndex(0),
            Update::SetForces(vec![])
        )));
    }

    #[test]
    fn vehicle_rotates_chassis_and_turret() {
        // Given: mods/tests1 "Tank1" rotates chassis at 30°/s, turret at 45°/s
        let world = world(|mod_| {
            let mut tank = tank(mod_);
            tank.chassis_target = Some(Direction::EST);
            tank.turret_target = Some(Direction::NORTH);
            tank
        });

        // When: 1 second
        let updates = step(
            &world,
            &Indexes::new(&world, &world.w),
            &world.vehicles().with_ids(),
            &world.w,
            &world.mod_,
            1.0,
        );

        // Then
        assert_eq!(updates.len(), 2);
        let mut chassis = None;
        let mut turret = None;
        for update in updates {
            match update {
                runner::update::Update::UpdateVehicle(_, Update::SetChassisOrientation(d)) => {
                    chassis = Some(d)
                }
                runner::update::Update::UpdateVehicle(_, Update::SetTurretDirection(d)) => {
                    turret = Some(d)
                }
                _ => unreachable!(),
            }
        }
        let angle = |d: Direction| Direction::SOUTH.angle_to(&d).abs().to_degrees();
        // Turret is carried by chassis rotation (30°) and rotates itself (45°)
        assert!((angle(chassis.unwrap()) - 30.).abs() < 1e-3);
        assert!((angle(turret.unwrap()) - 75.).abs() < 1e-3);
    }

    #[test]
    fn write_vehicle_position_updates_indexes_change() {
        // Given
        let mut world = world(tank);

        // When
        let (outputs, effects) = write(
            &mut world,
            Update::SetPosition(WorldVec3::new(250., 300., 0.)),
            VehicleIndex(0),
        );

        // Then
        assert_eq!(
            world.vehicle(VehicleIndex(0)).position,
            WorldVec3::new(250., 300., 0.)
        );
        assert_eq!(outputs.len(), 1);
        let [
            index::Effect::Vehicle(
                VehicleIndex(0),
                index::VehicleEffect::Placement { now, before, .. },
            ),
        ] = effects.as_slice()
        else {
            panic!("Expected one vehicle placement effect");
        };
        assert_eq!(now.0, WorldVec2::new(250., 300.));
        assert_eq!(before.0, WorldVec2::new(250., 250.));

        // When: no move
        let (_, effects) = write(&mut world, Update::SetReloadProgress(0.5), VehicleIndex(0));
        assert!(effects.is_empty());
    }
}
