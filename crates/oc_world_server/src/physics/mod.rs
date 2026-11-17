use std::time::{Duration, Instant};

use crate::runner::update::Update as Update_;
use derive_more::Constructor;
use oc_geo::{
    Geo, UpdateGeo,
    region::{Region, RegionXy, WorldRegionIndex},
    tile::{TileXy, WorldTileIndex},
};
use oc_individual::IndividualIndex;
pub use oc_physics::effect::Effect;
use oc_physics::{Event, Force, Physic, TickProgress, UpdatePhysic, update::Update};
use oc_projectile::ProjectileId;
use oc_root::{Client, WcfgInto, WorldConfig, geo::WorldVec3};
use oc_utils::collections::WithIds;
use oc_world::World;
use rustc_hash::FxHashMap;

use crate::{
    index::{self, IntoIndexEffect},
    network::{IntoNetworkForgot, IntoNetworkInsert, IntoNetworkUpdate},
    routing::Listening,
    state::ObjectId,
    utils::{
        context::Context,
        subject::{IntoSubject, IntoSubjectMut},
    },
};

pub mod collision;

type StepForResult<I> = (Vec<(I, Vec<Update>)>, Vec<Event<ObjectId>>);

#[derive(Constructor)]
pub struct Processor<'x, E: Client> {
    ctx: &'x Context<E>,
}

impl<'x, E: Client> Processor<'x, E> {
    pub fn step(&self, i: usize) -> Vec<crate::runner::update::Update> {
        tracing::trace!(name = "physics-step", i = i);

        let (subject_upds, subject_events) = self.step_for(i, |w| w.individuals().with_ids());
        let subject_effects = self.write(subject_upds);

        #[cfg(feature = "tracker")]
        self.track_subject_effects(&subject_effects);

        let (projectile_upds, projectile_events) = self.step_for(i, |w| w.projectiles().with_ids());
        let projectile_effects = self.write(projectile_upds);

        self.apply(subject_effects);
        self.apply(projectile_effects);

        let mut updates = vec![];
        updates.extend(self.react(subject_events));
        updates.extend(self.react(projectile_events));
        updates
    }

    #[cfg(feature = "tracker")]
    fn track_subject_effects(
        &self,
        subject_effects: &Vec<(IndividualIndex, (WorldRegionIndex, Option<Effect>), Update)>,
    ) {
        let mut tracker = self.ctx.tracker.take();
        for (i, _, update) in subject_effects {
            if let Update::SetPosition(position, _) = update {
                tracker.individual_positions.push((*i, *position));
            }
        }
    }

    fn step_for<F, I, T>(&self, i: usize, all: F) -> StepForResult<I>
    where
        for<'a> F: Fn(&'a World) -> Vec<(I, &'a T)>,
        I: Copy + Into<ObjectId> + std::fmt::Debug,
        T: Physic + Geo + Region,
    {
        let world = self.ctx.state.world();
        let indexes = self.ctx.state.indexes();
        let all = all(&world);
        let size = (all.len() as f32 / self.ctx.cpus as f32).ceil() as usize;
        if size == 0 {
            return (vec![], vec![]);
        };
        let chunks = all.chunks(size).collect::<Vec<_>>();
        let chunk = chunks.get(i);
        let Some(chunk) = chunk else {
            return (vec![], vec![]);
        };

        // Move code (which must take world and indexes as ref because RwReadLockGuard lifetime)
        let collision_objects = |xy| {
            let tile = TileXy(xy);
            let i: WorldTileIndex = tile.into_(&self.ctx.state.w);
            let individuals = indexes.tile_individuals(i);
            // let tile = world.tile(tile);

            let mut objects = vec![];

            for i in individuals {
                let individual: &dyn Physic = world.individual(*i);
                objects.push((ObjectId::Individual(*i), individual));
            }

            if let Some(tile_) = world.tile(tile.into_(&self.ctx.state.w)) {
                let tile: &dyn Physic = tile_;
                objects.push((ObjectId::Tile(tile_.i), tile));
            }

            objects
        };

        // Move code (which must take world and indexes as ref because RwReadLockGuard lifetime)
        let proximity_objects = |xy| {
            let tile = TileXy(xy);
            let i: WorldTileIndex = tile.into_(&self.ctx.state.w);
            let individuals = indexes.proximity_individuals(i);

            let mut objects = vec![];
            for i in individuals {
                let individual: &dyn Physic = world.individual(*i);
                objects.push((ObjectId::Individual(*i), individual));
            }

            objects
        };

        let mut events = vec![];
        let updates = chunk
            .iter()
            .map(|(i, subject)| {
                let (position, forces, events_) = oc_physics::step(
                    &self.ctx.state.w,
                    self.ctx.state.w.physics_tick().period() * self.ctx.state.w.speed(),
                    &self.ctx.state._mod,
                    (*i, *subject),
                    collision_objects,
                    proximity_objects,
                    self.ctx.state.w.ignore_firsts_physics_pixels() as usize,
                    "server"
                );
                tracing::trace!(name="physics-subject", i=?i, position=?position, forces=?forces);
                let updates = changes(&self.ctx.state.w, i, *subject, position, &forces);

                tracing::trace!(name = "physics-step-for", i = ?i, updates=?updates, events=events_.len());

                events.extend(events_);
                (*i, updates)
            })
            .collect::<Vec<_>>();

        #[cfg(feature = "tracker")]
        {
            self.ctx.tracker.take().physics.extend(events.clone());
        }

        (updates, events)
    }

    fn write<I, T>(
        &self,
        subjects: Vec<(I, Vec<Update>)>,
    ) -> Vec<(I, (WorldRegionIndex, Option<Effect>), Update)>
    where
        I: IntoSubjectMut<T> + std::fmt::Debug + Copy,
        T: Clone + Physic + UpdatePhysic + Geo + UpdateGeo + Region,
    {
        // One write lock for the whole batch for perf purpose
        let mut world = self.ctx.state.world_mut();
        subjects
            .into_iter()
            .flat_map(|(i, updates)| updates.into_iter().map(move |update| (i, update)))
            .filter_map(|(i, update)| {
                let Some(subject) = i.subject_mut(&mut world) else {
                    return None; // TODO: its possible ? What to do ? Simply log ?
                };
                let effect = write(&update, subject);

                Some((i, effect, update))
            })
            .collect()
    }

    fn apply<'a, I, T>(&self, effects: Vec<(I, (WorldRegionIndex, Option<Effect>), Update)>)
    where
        I: Copy + IntoSubject<T> + IntoNetworkUpdate + IntoIndexEffect<Effect> + std::fmt::Debug,
        T: 'a + IntoNetworkInsert<I> + IntoNetworkForgot<I>,
    {
        let mut index_effects = vec![];
        let mut region_changes = vec![];

        for (i, (region, effect), update) in effects {
            // Broadcast the update
            let filter = Listening::Regions(vec![region]);
            let messages = vec![i.to_network_update(update)];
            self.ctx.broadcast(filter, messages);

            if let Some(effect) = effect {
                index_effects.push(i.to_index_effect(effect.clone()));
                if let Effect::Region { before, after } = effect {
                    region_changes.push((i, before, after));
                }
            }
        }

        // Update indexes with one write lock for the whole batch (perf)
        if !index_effects.is_empty() {
            let mut indexes = self.ctx.state.indexes_mut();
            for effect in index_effects {
                indexes.react(effect, &self.ctx.state.w);
            }
        }

        // Broadcast to new listeners if required, with one read lock for the whole batch (perf)
        if !region_changes.is_empty() {
            let world = self.ctx.state.world();
            for (i, before, after) in region_changes {
                let Some(subject) = i.to_subject(&world) else {
                    continue; // TODO: its possible ? What to do ? Simply log ?
                };

                tracing::trace!(name="subject-update-write-broadcast-insert", i=?i);
                let filter = Listening::EnterBorder(before, after);
                let messages = vec![subject.to_network_insert(i)];
                self.ctx.broadcast(filter, messages);

                tracing::trace!(name="subject-update-write-broadcast-forgot", i=?i);
                let filter = Listening::ExitBorder(before, after);
                let messages = vec![subject.to_network_forgot(i)];
                self.ctx.broadcast(filter, messages);
            }
        }
    }

    fn react(&self, events: Vec<Event<ObjectId>>) -> Vec<crate::runner::update::Update> {
        let w = &self.ctx.state.w;
        let mut updates = vec![];
        // Consider only on proximity event par physics tick
        let mut proximity = FxHashMap::default();

        for event in events {
            tracing::trace!(name="physics-event", event=?event);

            if let Event::Collision(ObjectId::Projectile(projectile_id), _, progress) = event
                && let Some(projectile) = self.ctx.state.world().projectile(&projectile_id)
            {
                use oc_network::ToClient;

                let position = projectile.position();
                let messages = vec![ToClient::Collision(position, self.lag(progress))];
                self.ctx.broadcast(Listening::Any, messages);
            }

            match event {
                Event::NoTile(id) => match id {
                    ObjectId::Individual(_) | ObjectId::Tile(_) => {}
                    ObjectId::Projectile(id) => {
                        updates.push(crate::runner::update::Update::RemoveProjectile(id));
                    }
                },
                Event::Collision(a, b, progress) => match (a, b) {
                    (ObjectId::Individual(_), ObjectId::Individual(_))
                    | (ObjectId::Individual(_), ObjectId::Projectile(_))
                    | (ObjectId::Projectile(_), ObjectId::Projectile(_))
                    | (ObjectId::Individual(_), ObjectId::Tile(_))
                    | (ObjectId::Tile(_), ObjectId::Individual(_))
                    | (ObjectId::Tile(_), ObjectId::Projectile(_))
                    | (ObjectId::Tile(_), ObjectId::Tile(_)) => {}
                    (ObjectId::Projectile(projectile_id), ObjectId::Individual(individual_i)) => {
                        let lag = self.lag(progress);
                        updates.extend(
                            self.gunshot(projectile_id, individual_i)
                                .into_iter()
                                .map(|update| delay_sound(update, lag)),
                        );
                    }
                    (ObjectId::Projectile(id), ObjectId::Tile(tile)) => {
                        let lag = self.lag(progress);
                        updates.extend(
                            self.impact_sound(id, tile)
                                .map(|update| delay_sound(update, lag)),
                        );
                        updates.push(crate::runner::update::Update::RemoveProjectile(id));
                    }
                },
                Event::Proximity(kind, a, b, p) => {
                    // We care only about proximity when projectile move near individual
                    if let (ObjectId::Projectile(_), ObjectId::Individual(i)) = (a, b) {
                        proximity.insert(i, (kind, p.into()));
                    }
                    //
                }
                // No more consequences than no more forces
                Event::WantStop(_, _) => {}
            }
        }

        // Proximity to updates
        let world = self.ctx.state.world();
        let proximity = proximity
            .iter()
            .map(|(i, (kind, position))| {
                let individual = world.individual(*i);
                let suppress = self.increase_suppress(
                    w,
                    individual.suppress,
                    *kind,
                    individual.position,
                    *position,
                );
                crate::runner::update::Update::UpdateIndividual(
                    *i,
                    oc_individual::Update::SetSuppress(suppress),
                )
            })
            .collect::<Vec<_>>();
        updates.extend(proximity);

        updates
    }

    fn lag(&self, progress: TickProgress) -> Duration {
        self.ctx
            .state
            .w
            .physics_tick()
            .interval()
            .mul_f32(progress.0)
    }

    fn impact_sound(
        &self,
        id: ProjectileId,
        tile: WorldTileIndex,
    ) -> Option<crate::runner::update::Update> {
        if fastrand::f32() >= self.ctx.state.w.impact_sound_chance() {
            return None;
        }
        let world = self.ctx.state.world();
        let projectile = world.projectile(&id)?;
        let tile = world.tile(tile)?;
        let mod_ = self.ctx.state._mod();
        let sound = mod_.pick_impact_sound(mod_.nature(tile.nature).impact)?;

        Some(crate::runner::update::Update::PlaySoundWithGeo(
            sound,
            projectile.position(),
            projectile.region(),
            oc_physics::fx::SoundBus::Effects,
        ))
    }

    fn increase_suppress(
        &self,
        w: &WorldConfig,
        suppress: oc_root::behavior::Suppress,
        kind: oc_physics::ProximityKind,
        position_reference: WorldVec3,
        position: WorldVec3,
    ) -> oc_root::behavior::Suppress {
        let (maximum_distance, increment) = match kind {
            oc_physics::ProximityKind::Fly => w.proximity_projectile_fly_tick_increase_value(),
            oc_physics::ProximityKind::Impact => {
                w.proximity_projectile_impact_tick_increase_value()
            }
        };
        let maximum_distance = maximum_distance.pixels(w);

        if maximum_distance <= 0.0 {
            return suppress;
        }

        let event_distance = position_reference.distance(position);

        // When further than maximum distance, suppress value not modified
        if event_distance > maximum_distance {
            return suppress;
        }

        let proximity = maximum_distance - event_distance;
        let factor = proximity / maximum_distance;
        let value = (increment.0 as f32 * factor).round() as u8;
        oc_root::behavior::Suppress(suppress.0.saturating_add(value))
    }
}

pub fn changes<I, T>(
    w: &WorldConfig,
    i: I,
    subject: &T,
    position: WorldVec3,
    forces: &Vec<Force>,
) -> Vec<Update>
where
    I: std::fmt::Debug,
    T: Physic + Geo + Region,
{
    tracing::trace!(name="physics-changes", i=?i, position=?position, forces=?forces);
    let mut updates = vec![];

    // TODO: to improve perfs, maybe these updates sould be known at physics::step ?
    if subject.position(w) != position {
        updates.push(Update::SetPosition(position, subject.position(w)));

        let tile: TileXy = position.into_(w);
        let region: RegionXy = tile.into_(w);
        let tile: WorldTileIndex = tile.into_(w);
        let region: WorldRegionIndex = region.into_(w);

        if subject.tile() != tile {
            updates.push(Update::SetTile(tile, subject.tile()));
        }

        if subject.region() != region {
            updates.push(Update::SetRegion(region, subject.region()));
        }
    }

    for force in subject.forces(w) {
        if !forces.contains(force) {
            updates.push(Update::RemoveForce(force.clone()));
        }
    }

    for force in forces {
        if !subject.forces(w).contains(force) {
            updates.push(Update::PushForce(force.clone()));
        }
    }

    tracing::trace!(name="physics-individual-updates", i=?i, updates=?updates);

    updates
}

pub fn write<T>(update: &Update, subject: &mut T) -> (WorldRegionIndex, Option<Effect>)
where
    T: Clone + Physic + UpdatePhysic + Geo + UpdateGeo + Region,
{
    let tile = subject.tile();
    let region = subject.region();

    let effect = match update {
        Update::SetPosition(position, position_) => {
            subject.set_position(*position);
            Some(Effect::Position {
                before: *position_,
                after: *position,
            })
        }
        Update::SetTile(tile_, _) => {
            subject.set_tile(*tile_);
            Some(Effect::Tile {
                _before: tile,
                _after: *tile_,
            })
        }
        Update::SetRegion(region_, _) => {
            subject.set_region(*region_);
            Some(Effect::Region {
                before: region,
                after: *region_,
            })
        }
        Update::PushForce(force) => {
            subject.push_force(force.clone());
            None
        }
        Update::RemoveForce(force) => {
            subject.remove_force(force);
            None
        }
        Update::SetVolumes(volume, _) => {
            subject.set_volumes(volume.clone());
            None
        }
    };

    (region, effect)
}

impl IntoIndexEffect<Effect> for IndividualIndex {
    fn to_index_effect(&self, effect: Effect) -> index::Effect {
        let effect = index::IndividualEffect::Physic(effect.clone());
        index::Effect::Individual(*self, effect)
    }
}

impl IntoIndexEffect<Effect> for ProjectileId {
    fn to_index_effect(&self, effect: Effect) -> index::Effect {
        let effect = index::ProjectileEffect::Physic(effect.clone());
        index::Effect::Projectile(*self, effect)
    }
}

impl From<IndividualIndex> for ObjectId {
    fn from(value: IndividualIndex) -> Self {
        ObjectId::Individual(value)
    }
}

impl From<ProjectileId> for ObjectId {
    fn from(value: ProjectileId) -> Self {
        ObjectId::Projectile(value)
    }
}

/// Sounds must be played at the real instant of the event, not at the end of the physics tick
fn delay_sound(update: Update_, lag: Duration) -> Update_ {
    match update {
        Update_::PlaySoundWithGeo(..) if !lag.is_zero() => {
            Update_::Schedule(Instant::now() + lag, Box::new(update))
        }
        update => update,
    }
}
