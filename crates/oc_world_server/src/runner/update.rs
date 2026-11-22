use oc_utils::index::AtIndex;
use oc_utils::percent::Percent;
use oc_vehicle::VehicleIndex;
use std::time::{Duration, Instant};

use crate::projectile;
use crate::routing::Listening;
use crate::schedule::Schedule;
use crate::utils::context::Context;
use oc_geo::region::{Region, WorldRegionIndex};
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::IndividualIndex;
use oc_individual::squad::SquadIndex;
use oc_mod::grenade::GrenadeIndex;
use oc_mod::shell::ShellIndex;
use oc_mod::sound::SoundIndex;
use oc_mod::{Mod, PickSound};
use oc_network::ToClient;
use oc_physics::fx;
use oc_projectile::NextProjectileId;
use oc_projectile::Projectile;
use oc_projectile::ProjectileId;
use oc_projectile::spawn::SpawnProjectile;
use oc_projectile::spawn::SpawnProjectiles;
use oc_root::battle::BattlePhase;
use oc_root::explosion::Explosion;
use oc_root::geo::WorldVec2;
use oc_root::geo::WorldVec3;
use oc_root::side::BySide;
use oc_root::{Client, WcfgFrom, WcfgInto, WorldConfig};
use oc_world::crater::Crater;
use oc_world::flag::Flags;
use oc_world::morale::Morale;
use oc_world::smoke::SmokeCloud;
use oc_world::support::PendingSupport;
use oc_world::tile::AlteredTile;
use oc_world::visibility::Visibilities;
use oc_world::visibility::Visibility;
use rustc_hash::FxHashMap;

#[derive(Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Update {
    Schedule(Instant, Box<Update>),
    SpawnProjectiles(SpawnProjectiles),
    SpawnProjectile(SpawnProjectile, bool), // bool = fx
    /// Insert an already built projectile (like a gun shell)
    InsertProjectile(Projectile),
    RemoveProjectile(ProjectileId),
    UpdateIndividual(IndividualIndex, oc_individual::Update),
    /// Visibilities on opposite individuals and vehicles
    UpdateVisibilities(
        Vec<(IndividualIndex, IndividualIndex, Visibility)>,
        Vec<(IndividualIndex, VehicleIndex, Visibility)>,
    ),
    UpdateSquad(SquadIndex, oc_individual::squad::Update),
    UpdateVehicle(VehicleIndex, oc_vehicle::Update),
    SetBattlePhase(BattlePhase),
    UpdateFlags(Flags),
    UpdateMorale(Morale),
    UpdateElapsed(Duration, Duration), // elapsed, total duration
    PlaySoundWithGeo(SoundIndex, WorldVec3, WorldRegionIndex, fx::SoundBus),
    AddPendingSupport(PendingSupport),
    StartPendingSupport(PendingSupport),
    RealizeSupport(PendingSupport),
    RemovePendingSupport(PendingSupport),
    SpawnExplosion(Explosion, WorldVec2, ExplosionOrigin),
    AlterTiles(Vec<AlteredTile>),
    AddCrater(Crater),
    AddSmokeCloud(SmokeCloud),
    UpdateSmoke,
    RemoveDecor(Vec<WorldTileIndex>),
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExplosionOrigin {
    Shell(ShellIndex),
    Grenade(GrenadeIndex),
}

impl ExplosionOrigin {
    pub fn is_smoke(&self, mod_: &Mod) -> bool {
        matches!(
            self,
            ExplosionOrigin::Grenade(grenade) if mod_.grenade(*grenade).nature == oc_mod::grenade::GrenadeNature::Smoke
        )
    }
}

/// Apply updates in order. Consecutive individuals and squads updates only write world, so they
/// are applied under one world write lock instead of one lock by update (for perf).
pub fn update_all<E: Client>(ctx: &Context<E>, updates: Vec<Update>) {
    let writes_world =
        |u: &Update| matches!(u, Update::UpdateIndividual(..) | Update::UpdateSquad(..));
    let mut updates = updates.into_iter().peekable();

    while let Some(update_) = updates.next() {
        if !writes_world(&update_) {
            update(ctx, update_);
            continue;
        }

        // Gather the run of consecutive updates writing world, starting by `first`
        let mut run = vec![update_];
        while let Some(next) = updates.next_if(writes_world) {
            run.push(next);
        }
        write_world(ctx, run);
    }
}

/// Apply individuals and squads updates under one world write lock, then broadcast the results
fn write_world<E: Client>(ctx: &Context<E>, run: Vec<Update>) {
    let mut effects = vec![];
    let outputs = {
        let mut world = ctx.state.world_mut();
        run.into_iter()
            .flat_map(|update| match update {
                Update::UpdateIndividual(i, update) => {
                    #[cfg(feature = "tracker")]
                    ctx.tracker.take().individuals.push((i, update.clone()));
                    let (outputs, effects_) =
                        crate::individual::update::write(&mut world, update, i);
                    effects.extend(effects_);
                    outputs
                }
                Update::UpdateSquad(i, update) => {
                    super::State::<E>::write_squad(&mut world, i, update)
                }
                _ => unreachable!("Only individuals and squads updates are gathered"),
            })
            .collect::<Vec<_>>()
    };
    ctx.state.react_indexes(effects);

    // World lock is released before broadcast
    for (filter, messages) in outputs {
        ctx.broadcast(filter, messages);
    }
}

pub fn update<E: Client>(ctx: &Context<E>, update: Update) {
    let state = &ctx.state;
    tracing::trace!(name="runner-update", update=?update);

    for (filter, messages) in match update {
        Update::Schedule(instant, update) => state.schedule(instant, *update),
        Update::SpawnProjectiles(spawns) => state.spawn_projectiles(spawns),
        Update::SpawnProjectile(spawn, fx) => state.spawn_projectile(spawn, fx),
        Update::InsertProjectile(projectile) => state.insert_projectile(projectile, vec![]),
        Update::RemoveProjectile(i) => state.remove_projectile(i),
        Update::UpdateIndividual(i, update) => {
            #[cfg(feature = "tracker")]
            {
                let mut tracker = ctx.tracker.take();
                tracker.individuals.push((i, update.clone()));
            }
            state.update_individual(i, update)
        }
        Update::UpdateVisibilities(individuals, vehicles) => {
            state.update_visibilities(individuals, vehicles)
        }
        Update::UpdateSquad(i, update) => state.update_squad(i, update),
        Update::UpdateVehicle(i, update) => state.update_vehicle(i, update),
        Update::SetBattlePhase(phase) => state.set_battle_phase(phase),
        Update::UpdateFlags(flags) => state.update_flags(flags),
        Update::UpdateMorale(morale) => state.update_morale(morale),
        Update::UpdateElapsed(elapsed, duration) => state.update_elapsed(elapsed, duration),
        Update::PlaySoundWithGeo(sound, position, region, bus) => {
            state.play_sound(sound, position, region, bus)
        }
        Update::AddPendingSupport(pending) => state.add_pending_support(pending),
        Update::StartPendingSupport(pending) => state.start_pending_support(pending),
        Update::RealizeSupport(pending) => state.realize_support(pending),
        Update::RemovePendingSupport(pending) => state.remove_pending_support(pending),
        Update::SpawnExplosion(explosion, position, origin) => {
            #[cfg(feature = "tracker")]
            ctx.tracker.take().explosions.push((position, origin));
            state.spawn_explosion(explosion, position, origin)
        }
        Update::AddCrater(crater) => {
            let (i, region) = {
                let mut world = state.world_mut();
                let position = crater.position;
                let tile = WorldTileIndex::from_(TileXy::from_(position, &world.w), &world.w);
                let region: WorldRegionIndex = tile.into_(&world.w);
                world.craters.push(crater.clone());
                (world.craters.len() - 1, region)
            };
            state.indexes_mut().insert_crater(i, region);
            let message = ToClient::Craters(region, vec![crater]);
            vec![(Listening::Regions(vec![region]), vec![message])]
        }
        Update::AddSmokeCloud(cloud) => {
            let mut world = state.world_mut();
            let smoke: Vec<_> = cloud
                .tiles
                .iter()
                .map(|(i, opacity)| (*i, (*opacity).max(world.smoke_at(*i))))
                .collect();
            world.set_smoke(&smoke);
            world.smoke_clouds.push(cloud);
            smoke_messages(&world.w, smoke)
        }
        Update::UpdateSmoke => {
            let mut world = state.world_mut();
            let smoke = world.update_smoke(Instant::now());
            smoke_messages(&world.w, smoke)
        }
        Update::End => {
            ctx.signal(super::Signal::End);
            vec![]
        }
        Update::RemoveDecor(tiles) => {
            let mut by_region: FxHashMap<WorldRegionIndex, Vec<WorldTileIndex>> =
                FxHashMap::default();

            {
                let mut world = state.world_mut();
                for i in &tiles {
                    let region: WorldRegionIndex = (*i).into_(&world.w);
                    by_region.entry(region).or_default().push(*i);
                }
                world.remove_decor(&tiles);
            }

            by_region
                .into_iter()
                .map(|(region, tiles)| {
                    let message = ToClient::RemovedDecor(region, tiles);
                    (Listening::Regions(vec![region]), vec![message])
                })
                .collect()
        }
        Update::AlterTiles(altered) => {
            let mut by_region: FxHashMap<WorldRegionIndex, Vec<AlteredTile>> = FxHashMap::default();

            {
                let mut world = state.world_mut();
                for altered in &altered {
                    let region: WorldRegionIndex = altered.i.into_(&world.w);
                    by_region.entry(region).or_default().push(altered.clone());
                }
                world.alter_tiles(altered);
            }

            by_region
                .into_iter()
                .map(|(region, altered)| {
                    let message = ToClient::AlteredTiles(region, altered);
                    (Listening::Regions(vec![region]), vec![message])
                })
                .collect()
        }
    } {
        tracing::trace!(name="runner-update-broadcast", filter=?filter, messages=?messages);
        ctx.broadcast(filter, messages);
    }
}

/// Tell listeners of regions of changed smoke opacities
fn smoke_messages(
    w: &WorldConfig,
    smoke: Vec<(WorldTileIndex, Percent)>,
) -> Vec<(Listening, Vec<ToClient>)> {
    let mut by_region: FxHashMap<WorldRegionIndex, Vec<(WorldTileIndex, Percent)>> =
        FxHashMap::default();
    for (i, opacity) in smoke {
        let region: WorldRegionIndex = i.into_(w);
        by_region.entry(region).or_default().push((i, opacity));
    }

    by_region
        .into_iter()
        .map(|(region, smoke)| {
            let message = ToClient::Smoke(region, smoke);
            (Listening::Regions(vec![region]), vec![message])
        })
        .collect()
}

impl<E: Client> super::State<E> {
    pub fn schedule(&self, instant: Instant, update: Update) -> Vec<(Listening, Vec<ToClient>)> {
        self.scheduled().push((instant, update));
        vec![]
    }

    fn update_individual(
        &self,
        i: IndividualIndex,
        update: oc_individual::Update,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let (outputs, effects) = {
            let mut world = self.world_mut();
            crate::individual::update::write(&mut world, update, i)
        };
        self.react_indexes(effects);
        outputs
    }

    fn update_vehicle(
        &self,
        i: VehicleIndex,
        update: oc_vehicle::Update,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let (outputs, effects) = {
            let mut world = self.world_mut();
            crate::vehicle::write(&mut world, update, i)
        };
        self.react_indexes(effects);
        outputs
    }

    /// Apply effects on indexes (world lock must not be held)
    pub fn react_indexes(&self, effects: Vec<crate::index::Effect>) {
        if effects.is_empty() {
            return;
        }
        let mut indexes = self.indexes_mut();
        for effect in effects {
            indexes.react(effect, &self.w);
        }
    }

    fn update_visibilities(
        &self,
        individuals: Vec<(IndividualIndex, IndividualIndex, Visibility)>,
        vehicles: Vec<(IndividualIndex, VehicleIndex, Visibility)>,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        // Be sure to release world write lock
        let visibilities = {
            let mut world = self.world_mut();
            let count = world.individuals().len();
            world.visibilities = Visibilities::empty(count);

            for (i1, i2, visibility) in individuals.clone() {
                *world.visibilities.at_mut(i1).at_mut(i2) = visibility;
            }
            individuals
        };

        self.emit_visibilities(visibilities, vehicles)
    }

    /// Distribute visibilities according to side
    fn emit_visibilities(
        &self,
        individuals: Vec<(IndividualIndex, IndividualIndex, Visibility)>,
        vehicles: Vec<(IndividualIndex, VehicleIndex, Visibility)>,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let world = self.world();
        let count = world.individuals().len(); // Prepare memory for more than side individuals in favor of cpu performance
        let mut sides = BySide::from_fn(|_| Vec::with_capacity(count));
        let mut sides_vehicles = BySide::from_fn(|_| Vec::with_capacity(vehicles.len()));

        for (i1, i2, visibility) in individuals {
            let watcher = world.individual(i1);
            sides[watcher.side].push((i1, i2, visibility));
        }
        for (i1, i2, visibility) in vehicles {
            let watcher = world.individual(i1);
            sides_vehicles[watcher.side].push((i1, i2, visibility));
        }

        sides
            .into_iter()
            .zip(sides_vehicles)
            .map(|((side, visibilities), (_, vehicles))| {
                (
                    Listening::Side(side),
                    vec![ToClient::UpdateVisibilities(visibilities, vehicles)],
                )
            })
            .collect()
    }

    fn update_squad(
        &self,
        i: SquadIndex,
        update: oc_individual::squad::Update,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        Self::write_squad(&mut self.world_mut(), i, update)
    }

    pub fn write_squad(
        world: &mut oc_world::World,
        i: SquadIndex,
        update: oc_individual::squad::Update,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let squad = world.squad_mut(i);

        squad.apply(&update);
        let update = match update {
            oc_individual::squad::Update::SetOrderPosition(_, _) => {
                oc_individual::squad::Update::SetOrders(squad.orders.clone())
            }
            update => update,
        };

        let update = oc_individual::network::Squad::Update(i, update);
        let update = ToClient::Squad(update);
        vec![(Listening::Side(squad.side), vec![update])]
    }

    fn spawn_projectiles(&self, spawns: SpawnProjectiles) -> Vec<(Listening, Vec<ToClient>)> {
        for (i, (instant, fx)) in spawns.schedule((&self.w, &self._mod)).iter().enumerate() {
            let direction = match spawns.directions.get(i) {
                Some(direction) => *direction,
                None => match spawns.directions.first() {
                    Some(direction) => *direction,
                    None => {
                        tracing::error!("No direction available in SpawnProjectiles");
                        return vec![];
                    }
                },
            };
            let spawn = SpawnProjectiles::from_spawns(&spawns, direction);
            self.schedule(*instant, Update::SpawnProjectile(spawn, *fx));
        }

        vec![]
    }

    fn spawn_projectile(
        &self,
        spawn: SpawnProjectile,
        fx: bool,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let projectile = {
            let world = self.world();
            let mod_ = world.mod_();
            projectile::Builder::new(&self.w, mod_, spawn.clone()).build()
        };

        let sound = fx.then(|| self._mod().pick_sound((spawn.weapon, spawn.shot)));
        let fx = sound.flatten().map(|sound| {
            let position = projectile.position();
            ToClient::Fx(fx::Fx::Audio(fx::Audio::PlayWithGeo(
                sound,
                position,
                fx::SoundBus::Fire,
            )))
        });

        self.insert_projectile(projectile, fx.into_iter().collect())
    }

    /// Insert `projectile` in world and broadcast it (with `messages`) to its region listeners
    fn insert_projectile(
        &self,
        projectile: Projectile,
        messages: Vec<ToClient>,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let i = self._ids.next_projectile_id();

        // Make both insert and update index at same clock to lock at the same time
        {
            let mut world = self.world_mut();
            let mut indexes = self.indexes_mut();
            let projectiles = world.projectiles_mut();

            projectiles.insert(i, projectile.clone());
            indexes.insert_projectile(i, &projectile);
        }

        // Broadcast the new projectile (TODO: normalize/refactor to not call loop manually ?)
        let region: WorldRegionIndex = projectile.region();
        let insert = oc_projectile::network::Projectile::Insert(i, projectile);
        let mut messages_ = vec![ToClient::Projectile(insert)];
        messages_.extend(messages);

        vec![(Listening::Regions(vec![region]), messages_)]
    }

    fn remove_projectile(&self, id: ProjectileId) -> Vec<(Listening, Vec<ToClient>)> {
        let projectile = {
            let mut world = self.world_mut();
            let mut indexes = self.indexes_mut();

            if let Some(projectile) = world.projectiles_mut().remove(&id) {
                indexes.remove_projectile(&id, &projectile);
                Some(projectile)
            } else {
                None
            }
        };

        if let Some(projectile) = projectile {
            let region: WorldRegionIndex = projectile.region();
            let insert = oc_projectile::network::Projectile::Forgot(id);
            let message = ToClient::Projectile(insert);
            vec![(Listening::Regions(vec![region]), vec![message])]
        } else {
            vec![]
        }
    }

    fn set_battle_phase(&self, phase: BattlePhase) -> Vec<(Listening, Vec<ToClient>)> {
        self.world_mut().set_phase(phase);
        vec![(Listening::Any, vec![ToClient::SetBattlePhase(phase)])]
    }

    fn update_flags(&self, flags: Flags) -> Vec<(Listening, Vec<ToClient>)> {
        self.world_mut().flags = flags.clone();
        vec![(Listening::Any, vec![ToClient::Flags(flags)])]
    }

    fn update_morale(&self, morale: Morale) -> Vec<(Listening, Vec<ToClient>)> {
        vec![(Listening::Any, vec![ToClient::Morale(morale)])]
    }

    fn update_elapsed(
        &self,
        elapsed: Duration,
        duration: Duration,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        vec![(Listening::Any, vec![ToClient::Elapsed(elapsed, duration)])]
    }

    fn play_sound(
        &self,
        sound: SoundIndex,
        position: WorldVec3,
        region: WorldRegionIndex,
        bus: fx::SoundBus,
    ) -> Vec<(Listening, Vec<ToClient>)> {
        let fx = ToClient::Fx(fx::Fx::Audio(fx::Audio::PlayWithGeo(sound, position, bus)));
        vec![(Listening::Regions(vec![region]), vec![fx])]
    }
}
