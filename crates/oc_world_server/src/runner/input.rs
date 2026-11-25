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
    network::IntoNetworkInsert, runner::update::Update, state::State, utils::subject::IntoSubject,
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
        set_leader_position(&self.state.world(), squad, position, Angle::zero())
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
    let places = Deployment::members_positions(
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
    if !policy.fits(&world.w, &world.mod_, tiles, zones, position) || !places.iter().all(|p| inside(*p)) {
        tracing::debug!(name="runner-input-set-leader-position-outside-spawn", squad=?squad, position=?position);
        return vec![];
    }

    // Update squad position first to let GUI use squad position when orders refresh
    let set_position = oc_individual::squad::Update::SetPosition(position);
    let mut updates = vec![Update::UpdateSquad(squad, set_position)];

    let tile_i = WorldTileIndex::from_(position, &world.w);
    let vehicle_position = position.extend(world.tile_z(&world.w, tile_i));
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
    }
    updates
}
