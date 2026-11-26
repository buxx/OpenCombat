use std::sync::{Arc, mpsc::Sender};

use derive_more::Constructor;
use oc_deployment::deployment::Deployments as Deployment;
use oc_deployment::deployment::place::PlacementPolicy;
use oc_geo::region::WorldRegionIndex;
use oc_geo::tile::WorldTileIndex;
use oc_individual::squad::SquadIndex;
use oc_mod::Mod;
use oc_network::{SquadMessage, ToClient, ToServer};
use oc_projectile::spawn::SpawnProjectiles;
use oc_root::battle::BattlePhase;
use oc_root::identity::Identity;
use oc_root::{Client, WcfgFrom, geo::WorldVec2};
use oc_utils::d2::Angle;
use oc_utils::error::OkOrLogError;
use oc_utils::polygon::Polygon;
use oc_world::{
    support::{PendingSupport, SupportType},
    tile::IntoTiles,
};

use crate::{
    index::Indexes, individual, network::IntoNetworkInsert, runner::update::Update, state::State,
    utils::subject::IntoSubject,
};

#[derive(Constructor)]
pub struct Dealer<'a, E: Client> {
    state: &'a Arc<State<E>>,
    _mod: &'a Mod,
    output: &'a Sender<(E, ToClient)>,
    endpoint: E,
}

impl<'a, E: Client> Dealer<'a, E> {
    pub fn deal(&self, message: ToServer) -> Vec<Update> {
        tracing::trace!(name="runner-input", message=?message);

        match message {
            ToServer::RequestInit(indentity) => self.init(indentity),
            ToServer::ListenRegion(region) => self.listen_region(region),
            ToServer::ForgotRegion(region) => self.forgot_region(region),
            ToServer::Refresh => self.refresh(),
            ToServer::ExplodeProjectile(spawn) => self.spawn_projectiles(spawn),
            ToServer::Squad(squad, message) => self.squad_message(squad, message),
            ToServer::Fight => self.fight(),
            ToServer::RequestSupport(type_, position) => self.request_support(type_, position),
            #[cfg(feature = "debug")]
            ToServer::DebugExplosion(shell, position) => {
                let explosion = self._mod.shell(shell).inner().clone().into();
                let origin = super::update::ExplosionOrigin::Shell(shell);
                vec![Update::SpawnExplosion(explosion, position, origin)]
            }
            #[cfg(feature = "debug")]
            ToServer::DebugRequestSupport(side, type_, position) => {
                vec![Update::AddPendingSupport(PendingSupport::fresh(
                    side,
                    type_,
                    position,
                    std::time::Duration::ZERO,
                ))]
            }
            #[cfg(feature = "debug")]
            ToServer::DebugEndBattle => {
                crate::world::Processor::new(&self.state.world()).force_end()
            }
            #[cfg(feature = "debug")]
            ToServer::DebugKillSquad(side, nth) => self.kill_squad(side, nth),
            #[cfg(feature = "debug")]
            ToServer::DebugOwnFlags(side) => self.own_flags(side),
        }
    }

    #[cfg(feature = "debug")]
    fn own_flags(&self, side: oc_root::side::Side) -> Vec<Update> {
        let mut flags = self.state.world().flags.clone();
        let ownership = match side {
            oc_root::side::Side::A => oc_world::flag::Ownership::A,
            oc_root::side::Side::B => oc_world::flag::Ownership::B,
        };
        for (_, owner) in flags.iter_mut() {
            *owner = ownership.clone();
        }
        vec![Update::UpdateFlags(flags)]
    }

    #[cfg(feature = "debug")]
    fn kill_squad(&self, side: oc_root::side::Side, nth: u32) -> Vec<Update> {
        let world = self.state.world();
        let Some(squad) = world
            .squads()
            .iter()
            .filter(|s| s.side == side)
            .nth(nth as usize)
        else {
            tracing::warn!(name="runner-input-kill-squad-not-found", side=?side, nth=?nth);
            return vec![];
        };

        squad
            .members
            .iter()
            .flat_map(|i| crate::individual::death::die(&world, self._mod, *i))
            .collect()
    }

    fn init(&self, identity: Identity) -> Vec<Update> {
        {
            let mut listeners = self.state.listeners_mut();
            listeners.identify(self.endpoint.clone(), identity.clone());
        }

        let world = self.state.world();
        let resume = world.resume(&identity);
        let message = ToClient::WorldResume(resume);
        let message = (self.endpoint.clone(), message);

        self.output.send(message).ok_or_log();

        vec![]
    }

    fn listen_region(&self, region: WorldRegionIndex) -> Vec<Update> {
        tracing::trace!(name="dealer-listen-region", endpoint=?self.endpoint, region=?region);
        // Listeners lock must be released before refresh (which locks world): updates lock world
        // then listeners (to broadcast), so holding it here makes a deadlock.
        self.state
            .listeners_mut()
            .listen_region(&self.endpoint, region);
        self.refresh_region(region);
        vec![]
    }

    fn forgot_region(&self, region: WorldRegionIndex) -> Vec<Update> {
        tracing::trace!(name="dealer-forgot-region", endpoint=?self.endpoint, region=?region);
        let mut listeners = self.state.listeners_mut();
        listeners.forgot_region(&self.endpoint, region);
        vec![]
    }

    fn refresh(&self) -> Vec<Update> {
        // global refresh will send "non region" things (global game info, etc)
        vec![]
    }

    fn refresh_region(&self, region: WorldRegionIndex) {
        tracing::trace!(name="dealer-refresh-region", endpoint=?self.endpoint, region=?region);
        // Lock order is world then indexes everywhere: don't hold indexes while sending subjects
        // (which locks world), or a queued writer makes a deadlock.
        let (individuals, vehicles, projectiles, craters) = {
            let indexes = self.state.indexes();
            (
                indexes.region_individuals(region).clone(),
                indexes.region_vehicles(region).clone(),
                indexes.region_projectiles(region).clone(),
                indexes.region_craters(region).clone(),
            )
        };

        self.send_tiles(region, &craters);
        self.send_subjects(&individuals);
        self.send_subjects(&vehicles);
        self.send_subjects(&projectiles);
    }

    fn send_subjects<I, T>(&self, subjects: &Vec<I>)
    where
        I: Clone + IntoSubject<T>,
        T: IntoNetworkInsert<I>,
    {
        let world = self.state.world();

        for i in subjects {
            let Some(subject) = i.to_subject(&world) else {
                continue; // TODO: Possible ?
            };
            let subject = subject.to_network_insert(i.clone());
            let message = (self.endpoint.clone(), subject.into());
            self.output.send(message).ok_or_log();
        }
    }

    fn send_tiles(&self, region: WorldRegionIndex, craters: &[usize]) {
        let world = self.state.world();
        let tiles = region.to_tiles(&world);
        let tiles = ToClient::Tiles(region, tiles);
        let message = (self.endpoint.clone(), tiles);
        self.output.send(message).ok_or_log();

        let altered_tiles = world.region_altered_tiles(region);
        if !altered_tiles.is_empty() {
            let altered_tiles = ToClient::AlteredTiles(region, altered_tiles);
            let message = (self.endpoint.clone(), altered_tiles);
            self.output.send(message).ok_or_log();
        }

        let decor = world.region_decor(region);
        if !decor.is_empty() {
            let decor = ToClient::Decor(region, decor);
            let message = (self.endpoint.clone(), decor);
            self.output.send(message).ok_or_log();
        }

        let smoke = world.region_smoke(region);
        if !smoke.is_empty() {
            let smoke = ToClient::Smoke(region, smoke);
            let message = (self.endpoint.clone(), smoke);
            self.output.send(message).ok_or_log();
        }

        let craters: Vec<_> = craters.iter().map(|i| world.craters[*i].clone()).collect();
        if !craters.is_empty() {
            let craters = ToClient::Craters(region, craters);
            let message = (self.endpoint.clone(), craters);
            self.output.send(message).ok_or_log();
        }

        let wrecks = world.region_wrecks(region);
        if !wrecks.is_empty() {
            let wrecks = ToClient::Wrecks(region, wrecks);
            let message = (self.endpoint.clone(), wrecks);
            self.output.send(message).ok_or_log();
        }
    }

    fn spawn_projectiles(&self, spawns: SpawnProjectiles) -> Vec<Update> {
        tracing::trace!(name="runner-input-spawn-projectiles", spawn=?spawns);
        vec![Update::SpawnProjectiles(spawns)]
    }

    fn squad_message(&self, squad: SquadIndex, message: SquadMessage) -> Vec<Update> {
        match message {
            SquadMessage::SetOrders(orders) => {
                vec![Update::UpdateSquad(
                    squad,
                    oc_individual::squad::Update::SetOrders(orders),
                )]
            }
            SquadMessage::SetPositionOrderPosition(index, position) => {
                vec![Update::UpdateSquad(
                    squad,
                    oc_individual::squad::Update::SetOrderPosition(index, position),
                )]
            }
            SquadMessage::SetLeaderPosition(position) => self.set_leader_position(squad, position),
        }
    }

    fn fight(&self) -> Vec<Update> {
        // FIXME BS NOW: Against AI: fight immediately; Against human both must want fight to fight
        vec![Update::SetBattlePhase(BattlePhase::Fight)]
    }

    fn request_support(&self, type_: SupportType, position: WorldVec2) -> Vec<Update> {
        let Some(identity) = self.state.listeners().identity(&self.endpoint).cloned() else {
            return vec![];
        };

        request_support(&self.state.world(), identity.side, type_, position)
    }

    fn set_leader_position(&self, squad: SquadIndex, position: WorldVec2) -> Vec<Update> {
        set_leader_position(
            &self.state.world(),
            &self.state.indexes(),
            squad,
            position,
            Angle::zero(),
        )
    }
}

pub fn request_support(
    world: &oc_world::World,
    side: oc_root::side::Side,
    type_: SupportType,
    position: WorldVec2,
) -> Vec<Update> {
    tracing::debug!(name="runner-input-request-support", type_=?type_, side=?side, position=?position);

    // Wait is between following supports of same type (counted from the last realized one):
    // the request is accepted but its preparation is delayed by the rest of the wait
    let Some(support) = world.mod_.support(type_.name()) else {
        tracing::warn!(name="runner-input-request-support-unknown", type_=%type_);
        return vec![];
    };
    let wait = support.between_wait().div_f32(world.w.speed());

    // Minimum delay to wait to be able to start new support (this simulate the preparation time of
    // support to be able to fire again)
    let delay = world
        .support_realized
        .get(&(side, type_.clone()))
        .map(|last| wait.saturating_sub(last.elapsed()))
        .unwrap_or_default();

    // Prepare a fresh object
    let pending = PendingSupport::fresh(side, type_.clone(), position, delay);

    // Test if same support already pending
    let already = world
        .pending_supports
        .iter()
        .any(|p| p.side == side && p.type_ == type_);
    // If its a new support request and there is no more available, interupt the request
    if !already && world.available_support(side, &type_) == 0 {
        return vec![];
    }

    vec![Update::AddPendingSupport(pending)]
}

pub fn set_leader_position(
    world: &oc_world::World,
    indexes: &Indexes,
    squad: SquadIndex,
    position: WorldVec2,
    angle: Angle,
) -> Vec<Update> {
    let squad_ = world.squad(squad);
    let tiles = &world.tiles;
    let count = squad_.members.len();
    // Leader and all members must stay in zones owned by the squad side
    let zones = world.spawn_zones(squad_.side);
    // Squad vehicle (the leader one) is moved at leader position. Its crew stays inside it.
    let vehicle = world.individual(squad_.leader()).embarked.map(|(i, _)| i);
    let policy = vehicle
        .map(|vehicle| PlacementPolicy::from(world.vehicle(vehicle)))
        .unwrap_or(PlacementPolicy::Squad);
    let mut places = Deployment::members_positions(
        &world.w,
        &world.mod_,
        tiles,
        zones,
        position,
        angle,
        count,
        &policy,
    );
    let inside = |p: WorldVec2| zones.iter().any(|z| z.contains(p));
    if !policy.fits(&world.w, &world.mod_, tiles, zones, position)
        || !places.iter().all(|p| inside(*p))
    {
        tracing::debug!(name="runner-input-set-leader-position-outside-spawn", squad=?squad, position=?position);
        return vec![];
    }

    // Update squad position first to let GUI use squad position when orders refresh
    let set_position = oc_individual::squad::Update::SetPosition(position);
    let mut updates = vec![Update::UpdateSquad(squad, set_position)];

    let tile_i = WorldTileIndex::from_(position, &world.w);
    let vehicle_position = position.extend(world.tile_z(&world.w, tile_i));

    // Members (out of vehicle) take places of squad order formation (as leader distributes them), so
    // they don't have to move there once dropped. When these places leave spawn zones, keep
    // deployment places.
    let held = squad_.orders.first().filter(|order| order.holds());
    let leader_place = places.first().copied();
    let formation = match (vehicle, leader_place) {
        (None, Some(leader_place)) => {
            let tile_i = WorldTileIndex::from_(leader_place, &world.w);
            let reference = leader_place.extend(world.tile_z(&world.w, tile_i));
            individual::Processor::new(world, indexes, squad_.leader())
                .formation_places(reference)
                .filter(|formation| formation.iter().all(|(_, p)| inside(*p)))
        }
        _ => None,
    };
    if let Some(formation) = &formation {
        for (member, place) in formation {
            if let Some(k) = squad_.members.iter().position(|m| m == member) {
                places[k] = *place;
            }
        }
    }
    if let Some(vehicle) = vehicle {
        let set_position = oc_vehicle::Update::SetPosition(vehicle_position);
        updates.push(Update::UpdateVehicle(vehicle, set_position));
    }

    for (member, place) in squad_.members.iter().zip(places) {
        let embarked = world.individual(*member).embarked.map(|(i, _)| i);
        let position = match embarked {
            Some(embarked) if Some(embarked) == vehicle => vehicle_position,
            _ => {
                let tile_i = WorldTileIndex::from_(place, &world.w);
                place.extend(world.tile_z(&world.w, tile_i))
            }
        };
        let set_position = oc_individual::Update::SetPosition(position);
        updates.push(Update::UpdateIndividual(*member, set_position));

        // Already on their place: directly hold squad order (instead finish a previous move)
        let target = world.individual(*member).status.is_target();
        if let (true, Some(_), Some(order)) = (target, &formation, held) {
            let set_orders = oc_individual::Update::SetOrders(vec![order.clone()]);
            updates.push(Update::UpdateIndividual(*member, set_orders));
        }
    }
    updates
}

#[cfg(test)]
mod tests {
    use ::tests::{individual::TestIndividual, squad::TestSquad, world::TestWorld};
    use oc_individual::{IndividualIndex, order::Order};
    use oc_root::{
        WorldConfig,
        battle::BattlePhase,
        geo::{WorldVec2, WorldVec3},
    };
    use oc_utils::d2::{AlmostEqual, Angle, Direction};
    use oc_world::spawn::{SpawnZone, SpawnZoneName};

    use super::*;

    const TOLERANCE: f32 = 1.0;

    fn apply(world: &mut oc_world::World, updates: Vec<Update>) {
        for update in updates {
            match update {
                Update::UpdateIndividual(i, update) => {
                    individual::update::write(world, update, i);
                }
                Update::UpdateSquad(i, update) => world.squad_mut(i).apply(&update),
                update => panic!("Unexpected update {update:?}"),
            }
        }
    }

    // Hiding squad dropped during deployment: members are directly placed on their order formation
    // places, holding the order, so leader distributes them no move.
    #[test]
    fn test_set_leader_position_places_members_on_order_formation() {
        // Given
        let w = WorldConfig::new(100, 100);
        let order = Order::Hide(Direction::EST);
        let positions = [(100., 100.), (110., 100.), (120., 100.), (130., 100.)];
        let individuals = positions
            .iter()
            .map(|(x, y)| {
                let position = WorldVec3::new(*x, *y, 0.);
                TestIndividual::builder()
                    .position(position)
                    .build()
                    .make(&w)
            })
            .collect::<Vec<_>>();
        let squad = TestSquad::builder()
            .members((0..individuals.len()).map(IndividualIndex::from).collect())
            .position(WorldVec2::new(100., 100.))
            .orders(vec![order.clone()])
            .build()
            .make();
        let mut world = TestWorld::builder()
            .phase(BattlePhase::Deployment)
            .individuals(individuals)
            .squads(vec![squad])
            .build()
            .make(&w);
        world.side_spawn_zones.a = vec![SpawnZone {
            name: SpawnZoneName::new("A"),
            start: WorldVec2::new(10., 10.),
            points: vec![
                WorldVec2::new(0., 0.),
                WorldVec2::new(1600., 0.),
                WorldVec2::new(1600., 1600.),
                WorldVec2::new(0., 1600.),
            ],
        }];
        let squad = SquadIndex(0);

        // When
        let indexes = Indexes::new(&world, &w);
        let drop = WorldVec2::new(400., 400.);
        let updates = set_leader_position(&world, &indexes, squad, drop, Angle::zero());
        assert!(!updates.is_empty());
        apply(&mut world, updates);

        // Then: members stand on their formation places and hold squad order
        let indexes = Indexes::new(&world, &w);
        let leader = world.squad(squad).leader();
        let processor = individual::Processor::new(&world, &indexes, leader);
        let formation = processor
            .formation_places(world.individual(leader).position)
            .unwrap();
        assert_eq!(formation.len(), 3);
        for (member, place) in formation {
            let individual = world.individual(member);
            assert!(
                place.almost_equal(individual.position, TOLERANCE),
                "{member:?} at {:?} instead {place:?}",
                individual.position
            );
            assert_eq!(individual.orders, vec![order.clone()]);
        }

        // Then: leader distributes no move
        for update in processor.distribute() {
            if let Update::UpdateIndividual(i, oc_individual::Update::SetOrders(orders)) = update {
                assert_eq!(orders, vec![order.clone()], "{i:?}");
            }
        }
    }
}
