use glam::{Vec2, Vec3};
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{
    Individual, IndividualIndex, Update,
    behavior::{Behavior, Intent},
    order::Order,
    squad::{Squad, SquadIndex},
};
use oc_mod::{shell::ShellIndex, vehicle::CrewRole};
use oc_physics::Force;
use oc_projectile::Projectile;
use oc_root::{
    WcfgFrom,
    geo::{WorldVec2, WorldVec3},
};
use oc_utils::d2::Direction;
use oc_vehicle::{AIMED, LOADED, Vehicle, VehicleIndex};

use super::Processor;
use crate::runner;

/// Driver translates the vehicle only when chassis is oriented to its destination with this
/// tolerance (radians). Otherwise, it rotates the chassis first.
const CHASSIS_ALIGNED_TOLERANCE: f32 = 0.05;
/// Vehicle speed factor (of its maximum speed) when move
const MOVE_SPEED_FACTOR: f32 = 0.7;
/// Vehicle speed factor (of its maximum speed) when sneaking
const SNEAK_SPEED_FACTOR: f32 = 0.5;
/// Gunner fires only when turret is oriented to its target with this tolerance (radians)
const TURRET_ALIGNED_TOLERANCE: f32 = 0.01;

/// Crew role which must execute `order` when squad leader is in a vehicle. None when order concerns
/// nobody in particular (idle).
pub fn crew_role(order: &Order) -> Option<CrewRole> {
    match order {
        Order::Idle => None,
        Order::MoveTo(_) | Order::MoveFastTo(_) | Order::SneakTo(_) => Some(CrewRole::Driver),
        Order::Engage(_)
        | Order::Suppress(_)
        | Order::Grenade(_)
        | Order::Smoke(_)
        | Order::Defend(_)
        | Order::Hide(_) => Some(CrewRole::Gunner),
    }
}

/// Who must execute the squad order when the squad leader is embarked in a vehicle
enum Holder {
    /// Squad member
    Member(IndividualIndex),
    /// Nobody in the vehicle can execute it
    Nobody,
}

impl Processor<'_> {
    /// Squad member which must execute (and accomplish) the current squad order. It is the squad
    /// leader, except when it is embarked in a vehicle: then it depends on the order (see
    /// [`crew_role`]). None if nobody can.
    pub fn squad_order_holder(&self, squad: &Squad) -> Option<IndividualIndex> {
        match self.holder(squad) {
            Holder::Member(i) => Some(i),
            Holder::Nobody => None,
        }
    }

    fn holder(&self, squad: &Squad) -> Holder {
        let leader = squad.leader();
        let Some((vehicle, _)) = self.world.individual(leader).embarked else {
            return Holder::Member(leader);
        };
        let Some(role) = squad.orders.first().and_then(crew_role) else {
            return Holder::Member(leader);
        };

        match self.crew_member(squad, vehicle, role) {
            Some(member) => Holder::Member(member),
            None => Holder::Nobody,
        }
    }

    /// Operational squad member embarked in `vehicle` with `role`
    fn crew_member(
        &self,
        squad: &Squad,
        vehicle: VehicleIndex,
        role: CrewRole,
    ) -> Option<IndividualIndex> {
        squad.members.iter().copied().find(|member| {
            let member = self.world.individual(*member);
            member.embarked == Some((vehicle, role)) && member.status.is_target()
        })
    }

    /// Squad leader is embarked in a vehicle: squad order is given to the crew member owning the
    /// matching role. If there is no such member, squad order is dropped.
    pub fn distribute_in_vehicle(
        &self,
        squad_i: SquadIndex,
        squad: &Squad,
    ) -> Vec<runner::update::Update> {
        let mut updates = vec![];

        let holder = match self.holder(squad) {
            Holder::Member(member) => Some(member),
            Holder::Nobody => {
                tracing::trace!(name="individual-step-distribute-vehicle-drop-order", i=?self.i, order=?squad.orders.first());
                let orders = squad.orders.iter().skip(1).cloned().collect();
                updates.extend([
                    runner::update::Update::UpdateSquad(
                        squad_i,
                        oc_individual::squad::Update::Accomplished,
                    ),
                    runner::update::Update::UpdateSquad(
                        squad_i,
                        oc_individual::squad::Update::SetOrders(orders),
                    ),
                ]);
                None
            }
        };

        for member_i in &squad.members {
            let member = self.world.individual(*member_i);

            // Dead individuals must keep no order
            if !member.status.is_target() {
                continue;
            }

            let orders = match (holder, squad.orders.first()) {
                (Some(holder), Some(order)) if holder == *member_i => vec![order.clone()],
                _ => vec![],
            };

            if member.orders != orders {
                updates.push(runner::update::Update::UpdateIndividual(
                    *member_i,
                    Update::SetOrders(orders),
                ));
            }
        }

        updates
    }

    /// Embarked individual doesn't move or fire by itself: it operates the vehicle according to
    /// its crew role and orders.
    pub fn embarked(
        &self,
        individual: &Individual,
        vehicle_i: VehicleIndex,
        role: CrewRole,
    ) -> Vec<runner::update::Update> {
        let mut updates = vec![];
        let direction = individual.gesture.direction();

        // Individual body stays idle in the vehicle
        self.clear_individual(individual, &mut updates, direction);

        let vehicle = self.world.vehicle(vehicle_i);
        let (vehicle_updates, others) = match role {
            CrewRole::Driver => (self.drive(individual, vehicle), vec![]),
            CrewRole::Gunner => self.gun(individual, vehicle),
            CrewRole::Loader => (self.load(vehicle), vec![]),
            CrewRole::Commander | CrewRole::Operator | CrewRole::Passenger => (vec![], vec![]),
        };
        tracing::trace!(name="individual-step-embarked", i=?self.i, vehicle=?vehicle_i, role=?role, updates=?vehicle_updates, others=?others);

        updates.extend(
            vehicle_updates
                .into_iter()
                .map(|update| runner::update::Update::UpdateVehicle(vehicle_i, update)),
        );
        updates.extend(others);
        updates
    }

    fn clear_individual(
        &self,
        individual: &Individual,
        updates: &mut Vec<runner::update::Update>,
        direction: Direction,
    ) {
        let intent = Intent::Idle(direction);
        if individual.intent != intent {
            updates.push(self.individual_update(Update::SetIntent(intent)));
        }
        let behavior = Behavior::Idle(direction);
        if individual.behavior != behavior {
            updates.push(self.individual_update(Update::SetBehavior(behavior)));
        }
        if !individual.forces.is_empty() {
            updates.push(self.individual_update(Update::SetForces(vec![])));
        }
    }

    fn individual_update(&self, update: Update) -> runner::update::Update {
        runner::update::Update::UpdateIndividual(self.i, update)
    }

    /// Move orders: rotate chassis to the destination, then translate the vehicle to it
    fn drive(&self, individual: &Individual, vehicle: &Vehicle) -> Vec<oc_vehicle::Update> {
        let type_ = self.world.mod_.vehicle(vehicle.type_);
        let (destination, factor) = match individual.orders.first() {
            Some(Order::MoveTo(position)) => (Some(*position), 1.0),
            Some(Order::MoveFastTo(position)) => (Some(*position), MOVE_SPEED_FACTOR),
            Some(Order::SneakTo(position)) => (Some(*position), SNEAK_SPEED_FACTOR),
            _ => (None, 0.0),
        };

        let (chassis_target, forces, destination) = match destination {
            None => (None, vec![], None),
            Some(destination) => {
                let from: Vec2 = WorldVec2::from(vehicle.position).into();
                let direction = Direction::from_points2d(from, destination.into());
                let chassis = vehicle.chassis_direction;
                let aligned = chassis.angle_to(&direction).abs() <= CHASSIS_ALIGNED_TOLERANCE;

                match aligned {
                    true => {
                        let direction3 = WorldVec3::new(direction.x, direction.y, 0.);
                        let force = Force::Translation(direction3, type_.speed * factor);
                        (Some(direction), vec![force], Some(destination))
                    }
                    false => (Some(direction), vec![], None),
                }
            }
        };

        let mut updates = vec![];

        if vehicle.chassis_target != chassis_target {
            updates.push(oc_vehicle::Update::SetChassisTarget(chassis_target));
        }
        if vehicle.destination != destination {
            updates.push(oc_vehicle::Update::SetDestination(destination));
        }
        if vehicle.forces != forces {
            updates.push(oc_vehicle::Update::SetForces(forces));
        }

        updates
    }

    /// Fire orders: rotate the turret to the target, aim when turret is on it, then fire.
    /// Aiming restarts when firing is no longer possible (turret moved away from target,
    /// gun not loaded, etc).
    fn gun(
        &self,
        individual: &Individual,
        vehicle: &Vehicle,
    ) -> (Vec<oc_vehicle::Update>, Vec<runner::update::Update>) {
        let mut updates = self.orient_turret(individual, vehicle);
        let mut others = vec![];

        match self.fire_solution(individual, vehicle) {
            Some((target, shell)) => match self.aim(vehicle) {
                Ok(()) => {
                    let (vehicle_updates, projectile) = self.fire(vehicle, target, shell);
                    updates.extend(vehicle_updates);
                    updates.push(oc_vehicle::Update::SetAimProgress(0.));
                    others.push(projectile);
                }
                Err(progress) => updates.push(oc_vehicle::Update::SetAimProgress(progress)),
            },
            None => {
                if vehicle.aim_progress != 0. {
                    updates.push(oc_vehicle::Update::SetAimProgress(0.));
                }
            }
        }

        (updates, others)
    }

    // FIXME: super type Progress instead f32 (U8Progress in Progress f32)
    /// Progress aiming during one tick. Ok when aiming is finished, Err with the new progress
    /// otherwise.
    fn aim(&self, vehicle: &Vehicle) -> Result<(), f32> {
        let w = &self.world.w;
        let type_ = self.world.mod_.vehicle(vehicle.type_);
        let elapsed = w.individual_tick().period() * w.speed();
        let needed = (AIMED - vehicle.aim_progress).max(0.) * type_.aim_duration.0;

        match elapsed >= needed {
            true => Ok(()),
            false => Err(vehicle.aim_progress + elapsed / type_.aim_duration.0),
        }
    }

    /// Turret direction to reach the current order target
    fn turret_target(&self, individual: &Individual, vehicle: &Vehicle) -> Option<Direction> {
        let from: Vec2 = WorldVec2::from(vehicle.position).into();
        let toward = |to: WorldVec2| Some(Direction::from_points2d(from, to.into()));
        match individual.orders.first() {
            Some(Order::Defend(direction) | Order::Hide(direction)) => Some(*direction),
            _ => self.fire_target(individual).and_then(toward),
        }
    }

    /// Position where the gun must fire according to the current order
    fn fire_target(&self, individual: &Individual) -> Option<WorldVec2> {
        match individual.orders.first() {
            Some(Order::Engage(target)) => {
                Some(WorldVec2::from(self.world.individual(*target).position))
            }
            Some(Order::Suppress(position) | Order::Grenade(position) | Order::Smoke(position)) => {
                Some(*position)
            }
            Some(
                Order::Defend(_)
                | Order::Hide(_)
                | Order::Idle
                | Order::MoveTo(_)
                | Order::MoveFastTo(_)
                | Order::SneakTo(_),
            )
            | None => None,
        }
    }

    /// Rotate the turret to the target
    fn orient_turret(&self, individual: &Individual, vehicle: &Vehicle) -> Vec<oc_vehicle::Update> {
        if vehicle.turret_direction.is_none() {
            return vec![];
        }

        let target = self.turret_target(individual, vehicle);
        match vehicle.turret_target != target {
            true => vec![oc_vehicle::Update::SetTurretTarget(target)],
            false => vec![],
        }
    }

    /// Target position and shell to fire when the gun can fire (once aimed): fire order (suppress or
    /// engage), turret oriented to the target, gun loaded and compatible shell available.
    fn fire_solution(
        &self,
        individual: &Individual,
        vehicle: &Vehicle,
    ) -> Option<(WorldVec2, ShellIndex)> {
        if !matches!(
            individual.orders.first(),
            Some(Order::Suppress(_) | Order::Engage(_))
        ) {
            return None;
        }
        let target = self.fire_target(individual)?;
        let turret = vehicle.turret_direction?;
        if !vehicle.loaded() {
            return None;
        }

        let from: Vec2 = WorldVec2::from(vehicle.position).into();
        let direction = Direction::from_points2d(from, target.into());
        if turret.angle_to(&direction).abs() > TURRET_ALIGNED_TOLERANCE {
            return None;
        }

        let type_ = self.world.mod_.vehicle(vehicle.type_);
        let shell = vehicle
            .shells
            .iter()
            .find(|(shell, count)| *count > 0 && type_.compatible_shell.contains(shell))
            .map(|(shell, _)| *shell)?;

        Some((target, shell))
    }

    /// Fire `shell` to `target`: consume the shell, empty the gun and spawn the shell projectile.
    fn fire(
        &self,
        vehicle: &Vehicle,
        target: WorldVec2,
        shell: ShellIndex,
    ) -> (Vec<oc_vehicle::Update>, runner::update::Update) {
        let w = &self.world.w;
        let shells = vehicle
            .shells
            .iter()
            .map(|(shell_, count)| match *shell_ == shell {
                true => (*shell_, count.saturating_sub(1)),
                false => (*shell_, *count),
            })
            .collect();

        let type_ = self.world.mod_.vehicle(vehicle.type_);
        let from = WorldVec3::new(
            vehicle.position.x,
            vehicle.position.y,
            vehicle.position.z + type_.height.pixels(w),
        );
        let tile = WorldTileIndex::from_(TileXy::from_(target, w), w);
        let ground = self.world.tile(tile).map_or(0., |tile| tile.z_pixels(w));
        let target = target.extend(ground);
        let direction = Vec3::from(target - from).normalize_or_zero();
        let direction = WorldVec3::new(direction.x, direction.y, direction.z);
        let velocity = self.world.mod_.shell(shell).inner().velocity;
        let forces = vec![Force::Translation(direction, velocity)];
        let projectile = Projectile::Shell(oc_projectile::shell::Shell::new(
            shell,
            from,
            vehicle.side,
            forces,
            Some(self.i),
            w,
        ));
        tracing::debug!(name="individual-step-embarked-fire", i=?self.i, from=?from, target=?target, shell=?shell);

        (
            vec![
                oc_vehicle::Update::SetReloadProgress(0.),
                oc_vehicle::Update::SetShells(shells),
            ],
            runner::update::Update::InsertProjectile(projectile),
        )
    }

    /// Loader always loads the gun when it is not loaded
    fn load(&self, vehicle: &Vehicle) -> Vec<oc_vehicle::Update> {
        if vehicle.loaded() {
            return vec![];
        }

        let w = &self.world.w;
        let type_ = self.world.mod_.vehicle(vehicle.type_);
        let elapsed = w.individual_tick().period() * w.speed();
        let progress = (vehicle.reload_progress + elapsed / type_.reload_duration.0).min(LOADED);

        vec![oc_vehicle::Update::SetReloadProgress(progress)]
    }
}

#[cfg(test)]
mod tests {
    use ::tests::{individual::TestIndividual, squad::TestSquad, world::TestWorld};
    use oc_individual::{Individual, squad::Squad};
    use oc_mod::illustration::IllustrationIndex;
    use oc_root::{WorldConfig, physics::MetersSeconds, side::Side};
    use oc_world::World;

    use super::*;
    use crate::index::Indexes;

    const VEHICLE: VehicleIndex = VehicleIndex(0);
    /// Speed of mods/tests1 "Tank1"
    const TANK_SPEED: MetersSeconds = MetersSeconds(5.0);

    fn w() -> WorldConfig {
        WorldConfig::new(100, 100)
    }

    fn tank(w: &WorldConfig, chassis: Direction) -> Vehicle {
        let mod_ = TestWorld::builder().build().make(w).mod_;
        Vehicle::new(
            Side::A,
            mod_.vehicle_from_name("Tank1").unwrap().index(),
            IllustrationIndex(0),
            WorldVec3::new(250., 250., 0.),
            vec![],
            chassis,
            Some(chassis),
        )
    }

    /// World with a squad embarked in `vehicle` (with `crew` roles, first is the leader) plus
    /// `others` individuals (each in its own squad)
    fn tank_world(
        w: &WorldConfig,
        vehicle: Vehicle,
        crew: &[(CrewRole, Vec<Order>)],
        squad_orders: Vec<Order>,
        others: Vec<Individual>,
    ) -> World {
        let mut individuals: Vec<Individual> = crew
            .iter()
            .map(|(role, orders)| {
                let mut individual = TestIndividual::builder()
                    .position(vehicle.position)
                    .build()
                    .make(w)
                    .with_embarked(Some((VEHICLE, *role)));
                individual.orders = orders.clone();
                individual
            })
            .collect();
        let crew_squad = TestSquad::builder()
            .members((0..crew.len()).map(|i| IndividualIndex(i as u64)).collect())
            .orders(squad_orders)
            .build()
            .make();
        let mut squads: Vec<Squad> = vec![crew_squad];
        for other in others {
            let i = IndividualIndex(individuals.len() as u64);
            squads.push(TestSquad::builder().members(vec![i]).build().make());
            individuals.push(other);
        }

        TestWorld::builder()
            .individuals(individuals)
            .squads(squads)
            .vehicles(vec![vehicle])
            .build()
            .make(w)
    }

    fn step(world: &World, i: u64) -> Vec<runner::update::Update> {
        let indexes = Indexes::new(world, &world.w);
        Processor::new(world, &indexes, IndividualIndex(i)).step()
    }

    fn set_orders(i: u64, orders: Vec<Order>) -> runner::update::Update {
        runner::update::Update::UpdateIndividual(IndividualIndex(i), Update::SetOrders(orders))
    }

    fn vehicle_update(update: oc_vehicle::Update) -> runner::update::Update {
        runner::update::Update::UpdateVehicle(VEHICLE, update)
    }

    fn has_orders_update(updates: &[runner::update::Update], i: u64) -> bool {
        updates.iter().any(|u| {
            matches!(u, runner::update::Update::UpdateIndividual(i_, Update::SetOrders(_)) if i_.0 == i)
        })
    }

    #[test]
    fn leader_in_vehicle_gives_move_order_to_driver() {
        // Given
        let w = w();
        let order = Order::MoveTo(WorldVec2::new(250., 400.));
        let crew = [
            (CrewRole::Commander, vec![]),
            (CrewRole::Driver, vec![]),
            (CrewRole::Gunner, vec![]),
        ];
        let world = tank_world(
            &w,
            tank(&w, Direction::NORTH),
            &crew,
            vec![order.clone()],
            vec![],
        );

        // When
        let updates = step(&world, 0);

        // Then: only driver receive the order
        assert!(updates.contains(&set_orders(1, vec![order])));
        assert!(!has_orders_update(&updates, 0));
        assert!(!has_orders_update(&updates, 2));
    }

    #[test]
    fn leader_in_vehicle_gives_fire_orders_to_gunner() {
        let w = w();
        let crew = [
            (CrewRole::Commander, vec![]),
            (CrewRole::Driver, vec![]),
            (CrewRole::Gunner, vec![]),
        ];

        for order in [
            Order::Defend(Direction::EST),
            Order::Hide(Direction::EST),
            Order::Suppress(WorldVec2::new(400., 250.)),
            Order::Grenade(WorldVec2::new(400., 250.)),
            Order::Smoke(WorldVec2::new(400., 250.)),
        ] {
            // Given
            let world = tank_world(
                &w,
                tank(&w, Direction::NORTH),
                &crew,
                vec![order.clone()],
                vec![],
            );

            // When
            let updates = step(&world, 0);

            // Then
            assert!(updates.contains(&set_orders(2, vec![order])));
            assert!(!has_orders_update(&updates, 0));
            assert!(!has_orders_update(&updates, 1));
        }
    }

    #[test]
    fn leader_in_vehicle_drops_order_without_crew_for_it() {
        // Given: no driver
        let w = w();
        let crew = [(CrewRole::Commander, vec![]), (CrewRole::Gunner, vec![])];
        let orders = vec![
            Order::MoveTo(WorldVec2::new(250., 400.)),
            Order::Defend(Direction::EST),
        ];
        let world = tank_world(&w, tank(&w, Direction::NORTH), &crew, orders, vec![]);

        // When
        let updates = step(&world, 0);

        // Then: move order dropped
        assert!(updates.contains(&runner::update::Update::UpdateSquad(
            SquadIndex(0),
            oc_individual::squad::Update::SetOrders(vec![Order::Defend(Direction::EST)]),
        )));
        assert!(!has_orders_update(&updates, 1));
    }

    #[test]
    fn driver_rotates_chassis_before_moving() {
        // Given: tank facing est, driver ordered to move north
        let w = w();
        let crew = [
            (CrewRole::Commander, vec![]),
            (
                CrewRole::Driver,
                vec![Order::MoveTo(WorldVec2::new(250., 400.))],
            ),
        ];
        let world = tank_world(&w, tank(&w, Direction::EST), &crew, vec![], vec![]);

        // When
        let updates = step(&world, 1);

        // Then: chassis rotate to north, without translation
        assert!(
            updates.contains(&vehicle_update(oc_vehicle::Update::SetChassisTarget(Some(
                Direction::NORTH
            ))))
        );
        assert!(!updates.iter().any(|u| matches!(
            u,
            runner::update::Update::UpdateVehicle(_, oc_vehicle::Update::SetForces(_))
        )));
        // Driver individual itself doesn't move
        assert!(!updates.iter().any(|u| matches!(
            u,
            runner::update::Update::UpdateIndividual(_, Update::SetForces(f)) if !f.is_empty()
        )));
    }

    #[test]
    fn driver_moves_when_chassis_oriented() {
        // Given: tank facing north, driver ordered to move north
        let w = w();
        let destination = WorldVec2::new(250., 400.);
        let crew = [
            (CrewRole::Commander, vec![]),
            (CrewRole::Driver, vec![Order::MoveTo(destination)]),
        ];
        let world = tank_world(&w, tank(&w, Direction::NORTH), &crew, vec![], vec![]);

        // When
        let updates = step(&world, 1);

        // Then
        let force = Force::Translation(WorldVec3::new(0., 1., 0.), TANK_SPEED);
        assert!(updates.contains(&vehicle_update(oc_vehicle::Update::SetForces(vec![force]))));
        assert!(
            updates.contains(&vehicle_update(oc_vehicle::Update::SetDestination(Some(
                destination
            ))))
        );
    }

    #[test]
    fn driver_stops_vehicle_without_move_order() {
        // Given: moving tank, driver without order
        let w = w();
        let mut vehicle = tank(&w, Direction::NORTH);
        vehicle.forces = vec![Force::Translation(WorldVec3::new(0., 1., 0.), TANK_SPEED)];
        let crew = [(CrewRole::Commander, vec![]), (CrewRole::Driver, vec![])];
        let world = tank_world(&w, vehicle, &crew, vec![], vec![]);

        // When
        let updates = step(&world, 1);

        // Then
        assert!(updates.contains(&vehicle_update(oc_vehicle::Update::SetForces(vec![]))));
    }

    #[test]
    fn driver_accomplishes_squad_move_order() {
        // Given: vehicle reached the squad move order position
        let w = w();
        let order = Order::MoveTo(WorldVec2::new(250., 250.));
        let crew = [
            (CrewRole::Commander, vec![]),
            (CrewRole::Driver, vec![order.clone()]),
        ];
        let world = tank_world(&w, tank(&w, Direction::NORTH), &crew, vec![order], vec![]);

        // When
        let updates = step(&world, 1);

        // Then: squad order is finished
        assert!(updates.contains(&runner::update::Update::UpdateSquad(
            SquadIndex(0),
            oc_individual::squad::Update::SetOrders(vec![]),
        )));
    }

    #[test]
    fn driver_accomplishes_squad_move_order_when_vehicle_stopped_near_destination() {
        // Given: vehicle stopped near (within its destination tolerance, but farther than the
        // individual position tolerance) the squad move order position
        let w = w();
        let order = Order::MoveTo(WorldVec2::new(
            250.,
            250. + oc_vehicle::DESTINATION_TOLERANCE - 0.5,
        ));
        let crew = [
            (CrewRole::Commander, vec![]),
            (CrewRole::Driver, vec![order.clone()]),
        ];
        let world = tank_world(&w, tank(&w, Direction::NORTH), &crew, vec![order], vec![]);

        // When
        let updates = step(&world, 1);

        // Then: individual and squad orders are finished
        assert!(updates.contains(&runner::update::Update::UpdateIndividual(
            IndividualIndex(1),
            Update::Accomplished,
        )));
        assert!(updates.contains(&runner::update::Update::UpdateSquad(
            SquadIndex(0),
            oc_individual::squad::Update::SetOrders(vec![]),
        )));
    }

    #[test]
    fn gunner_rotates_turret_to_engaged_target() {
        // Given: turret facing est, enemy at north
        let w = w();
        let enemy = TestIndividual::builder()
            .side(Side::B)
            .position(WorldVec3::new(250., 400., 0.))
            .build()
            .make(&w);
        let crew = [
            (CrewRole::Commander, vec![]),
            (CrewRole::Gunner, vec![Order::Engage(IndividualIndex(2))]),
        ];
        let world = tank_world(&w, tank(&w, Direction::EST), &crew, vec![], vec![enemy]);

        // When
        let updates = step(&world, 1);

        // Then
        assert!(
            updates.contains(&vehicle_update(oc_vehicle::Update::SetTurretTarget(Some(
                Direction::NORTH
            ))))
        );
    }

    /// Tank facing north, loaded, half aimed, with 2 shells of mods/tests1 "5/38 shell"
    /// (compatible with "Tank1"), gunner ordered to suppress `target`. mods/tests1 "Tank1" aim
    /// duration is 2 seconds (so, a tick of one second finishes aiming).
    fn firing_world(w: &WorldConfig, vehicle: impl FnOnce(Vehicle) -> Vehicle) -> World {
        let mod_ = TestWorld::builder().build().make(w).mod_;
        let shell = mod_.shell_from_name("5/38 shell").unwrap().index();
        let mut tank = tank(w, Direction::NORTH).with_shells(vec![(shell, 2)]);
        tank.aim_progress = 0.5;
        let vehicle = vehicle(tank);
        let crew = [
            (CrewRole::Commander, vec![]),
            (
                CrewRole::Gunner,
                vec![Order::Suppress(WorldVec2::new(250., 400.))],
            ),
        ];
        tank_world(w, vehicle, &crew, vec![], vec![])
    }

    /// Shell projectile spawned by `updates`
    fn shell_fired(updates: &[runner::update::Update]) -> Option<oc_projectile::shell::Shell> {
        updates.iter().find_map(|u| match u {
            runner::update::Update::InsertProjectile(Projectile::Shell(shell)) => {
                Some(shell.clone())
            }
            _ => None,
        })
    }

    fn aim_progress(updates: &[runner::update::Update]) -> Option<f32> {
        updates.iter().find_map(|u| match u {
            runner::update::Update::UpdateVehicle(_, oc_vehicle::Update::SetAimProgress(p)) => {
                Some(*p)
            }
            _ => None,
        })
    }

    #[test]
    fn gunner_aims_before_firing() {
        // Given: turret on target, aiming not started
        let w = w();
        let world = firing_world(&w, |mut vehicle| {
            vehicle.aim_progress = 0.;
            vehicle
        });

        // When: one second
        let updates = step(&world, 1);

        // Then: half aimed (2 seconds aim duration), not fired
        assert_eq!(shell_fired(&updates), None);
        assert!((aim_progress(&updates).unwrap() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn gunner_aiming_restarts_when_turret_leaves_target() {
        // Given: half aimed, but turret no more on target
        let w = w();
        let world = firing_world(&w, |mut vehicle| {
            vehicle.turret_direction = Some(Direction::EST);
            vehicle
        });

        // When
        let updates = step(&world, 1);

        // Then
        assert_eq!(aim_progress(&updates), Some(0.));
    }

    #[test]
    fn gunner_fires_when_turret_on_target() {
        // Given
        let w = w();
        let world = firing_world(&w, |vehicle| vehicle);
        let shell = world.vehicle(VEHICLE).shells[0].0;

        // When
        let updates = step(&world, 1);

        // Then: shell fired from the tank toward the target, gun emptied, aiming restarts, shell
        // consumed
        assert_eq!(aim_progress(&updates), Some(0.));
        let fired = shell_fired(&updates).unwrap();
        let from = world.vehicle(VEHICLE).position;
        assert_eq!(fired.shell, shell);
        assert_eq!((fired.position.x, fired.position.y), (from.x, from.y));
        let [Force::Translation(direction, _)] = fired.forces.as_slice() else {
            panic!("Shell must have one translation force");
        };
        let expected = (Vec2::new(250., 400.) - Vec2::new(from.x, from.y)).normalize();
        let direction = Vec2::new(direction.x, direction.y).normalize();
        assert!(direction.distance(expected) < 1e-4);
        assert!(updates.contains(&vehicle_update(oc_vehicle::Update::SetReloadProgress(0.))));
        assert!(
            updates.contains(&vehicle_update(oc_vehicle::Update::SetShells(vec![(
                shell, 1
            )])))
        );
    }

    #[test]
    fn gunner_does_not_fire_when_turret_not_on_target() {
        let w = w();
        let world = firing_world(&w, |mut vehicle| {
            vehicle.turret_direction = Some(Direction::EST);
            vehicle
        });

        let updates = step(&world, 1);

        assert_eq!(shell_fired(&updates), None);
    }

    #[test]
    fn gunner_does_not_fire_when_gun_not_loaded() {
        let w = w();
        let world = firing_world(&w, |vehicle| vehicle.with_reload_progress(0.5));

        let updates = step(&world, 1);

        assert_eq!(shell_fired(&updates), None);
    }

    #[test]
    fn gunner_does_not_fire_without_shells() {
        let w = w();
        let world = firing_world(&w, |mut vehicle| {
            vehicle.shells[0].1 = 0;
            vehicle
        });

        let updates = step(&world, 1);

        assert_eq!(shell_fired(&updates), None);
    }

    #[test]
    fn loader_loads_gun() {
        // Given: gun not loaded (mods/tests1 "Tank1" reload duration is 6 seconds)
        let w = w();
        let vehicle = tank(&w, Direction::NORTH).with_reload_progress(0.);
        let crew = [(CrewRole::Commander, vec![]), (CrewRole::Loader, vec![])];
        let world = tank_world(&w, vehicle, &crew, vec![], vec![]);

        // When
        let updates = step(&world, 1);

        // Then: one second of loading
        let progress = updates.iter().find_map(|u| match u {
            runner::update::Update::UpdateVehicle(_, oc_vehicle::Update::SetReloadProgress(p)) => {
                Some(*p)
            }
            _ => None,
        });
        assert!((progress.unwrap() - 1. / 6.).abs() < 1e-4);
    }

    #[test]
    fn loader_does_nothing_when_loaded() {
        let w = w();
        let crew = [(CrewRole::Commander, vec![]), (CrewRole::Loader, vec![])];
        let world = tank_world(&w, tank(&w, Direction::NORTH), &crew, vec![], vec![]);

        let updates = step(&world, 1);

        assert!(
            !updates
                .iter()
                .any(|u| matches!(u, runner::update::Update::UpdateVehicle(..)))
        );
    }
}
