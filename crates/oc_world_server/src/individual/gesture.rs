use std::time::{Duration, Instant};

use glam::Vec2;
use oc_geo::tile::{TileXy, WorldTileIndex};
use oc_individual::{
    BodyGesture, Gesture, HandsGesture, Individual, IndividualIndex, WeaponKind,
    behavior::Behavior, order::Order,
};
use oc_mod::{
    ammunition::AmmunitionIndex,
    grenade::{GrenadeNature, IndexedGrenade},
    weapons::IndexedWeapon,
};
use oc_root::{
    WcfgFrom,
    geo::{WorldVec2, WorldVec3},
    physics::Seconds,
    utils::U8Progress,
};
use oc_utils::{d2::Direction, let_some};

use super::{Processor, situation::Situation};
use crate::{
    runner::{self, update::ExplosionOrigin},
    visibility::{path_objects_at, visibility},
};

/// Duration of each step (reload, aim) of grenade throwing
const THROW_STEP_SECONDS: f32 = 1.0;

type Updates = Vec<runner::update::Update>;

impl Processor<'_> {
    pub fn gesture(
        &self,
        individual: &Individual,
        situation: &Situation,
        behavior: &Behavior,
    ) -> (Gesture, Updates) {
        let body = |gesture| (Gesture::body(gesture), vec![]);

        match behavior {
            Behavior::Idle(direction) => match situation.imply_hide() {
                true => body(BodyGesture::Prone(*direction)),
                false => body(BodyGesture::StandUp(*direction)),
            },
            Behavior::Walk(direction) => body(BodyGesture::Walking(*direction)),
            Behavior::Run(direction) => body(BodyGesture::Running(*direction)),
            Behavior::Crawl(direction) => body(BodyGesture::Crawling(*direction)),
            Behavior::Defend(direction) | Behavior::Hide(direction) => {
                body(BodyGesture::Prone(*direction))
            }
            Behavior::Engage(target) => self.engage_gesture(individual, situation, *target),
            Behavior::Suppress(target) => self.suppress_gesture(individual, situation, *target),
            Behavior::Grenade(target) => {
                self.throw_gesture(individual, *target, GrenadeNature::Frag)
            }
            Behavior::Smoke(target) => {
                self.throw_gesture(individual, *target, GrenadeNature::Smoke)
            }
        }
    }

    pub fn engage_gesture(
        &self,
        individual: &Individual,
        situation: &Situation,
        target: IndividualIndex,
    ) -> (Gesture, Updates) {
        let target_ = self.world.individual(target);
        let reference = individual.position.into();
        let direction = Direction::from_points3d(reference, target_.position.into());
        let_some!(
            visible = situation.visible(target),
            // Should not happen as engage gesture is used when target is visible
            return (Gesture::body(BodyGesture::Prone(direction)), vec![])
        );

        let (hands, updates) = match self.weapon(individual, target_) {
            // Individual own weapon adapted to this target
            Some((kind, weapon)) => self.hands(
                individual,
                weapon,
                kind.clone(),
                |weapon, ammunition, lag| {
                    let plus_z = target_.gesture.body.target_z().pixels(&self.world.w);
                    let target = WorldVec3::new(
                        target_.position.x,
                        target_.position.y,
                        target_.position.z + plus_z,
                    );
                    let opacity = visible.visibility.opacity;
                    self.spawn_projectile(
                        individual, weapon, ammunition, kind, target, opacity, lag,
                    )
                },
            ),
            // No weapon adapted to the target
            // FIXME: choose/try another target where weapon match ? do it where behavior chosen ?
            None => {
                tracing::trace!(name="individual-processor-engage-gesture-no-weapon-found", i=?self.i, target=?target);
                (HandsGesture::Idle, vec![])
            }
        };

        (
            Gesture::body(BodyGesture::Prone(direction)).with_hands(hands),
            updates,
        )
    }

    pub fn suppress_gesture(
        &self,
        individual: &Individual,
        _situation: &Situation,
        target: WorldVec2,
    ) -> (Gesture, Updates) {
        let w = &self.world.w;
        let mod_ = &self.world.mod_;
        let reference = Vec2::new(individual.position.x, individual.position.y);
        let direction = Direction::from_points2d(reference, Vec2::new(target.x, target.y));
        // TODO: mechanism to cache (perf) ?
        let tile_i = WorldTileIndex::from_(TileXy::from_(target, w), w);
        let_some!(
            target_tile = self.world.tile(tile_i),
            return (Gesture::body(BodyGesture::Prone(direction)), vec![])
        );
        let tile_z = target_tile.z_pixels(w);
        let at = |xy, z| path_objects_at(w, mod_, self.world, xy, z);
        let target_ = target.extend(tile_z);
        let modifier = oc_lov::Modifier::None(false);
        let visibility = visibility(w, at, individual.position, target_, modifier);
        tracing::trace!(name="individual-processor-suppress-gesture-visibility", i=?self.i, target=?target, tile_z=tile_z, visibility=?visibility);

        let (hands, updates) = match self.primary_weapon(individual, target) {
            // Individual own weapon adapted to this target
            Some(weapon) => self.hands(
                individual,
                weapon,
                WeaponKind::Primary,
                |weapon, ammunition, lag| {
                    let target = WorldVec3::new(target.x, target.y, tile_z);
                    let opacity = visibility.opacity;
                    self.spawn_projectile(
                        individual,
                        weapon,
                        ammunition,
                        WeaponKind::Primary,
                        target,
                        opacity,
                        lag,
                    )
                },
            ),
            // No weapon adapted to the target
            None => {
                tracing::trace!(name="individual-processor-suppress-gesture-no-weapon-found", i=?self.i, target=?target);
                (HandsGesture::Idle, vec![])
            }
        };

        (
            Gesture::body(BodyGesture::Prone(direction)).with_hands(hands),
            updates,
        )
    }

    /// Throw a grenade/smoke on target: reload, aim, then throw.
    pub fn throw_gesture(
        &self,
        individual: &Individual,
        target: WorldVec2,
        nature: GrenadeNature,
    ) -> (Gesture, Updates) {
        let w = &self.world.w;
        let tick = w.individual_tick();
        let reference = Vec2::new(individual.position.x, individual.position.y);
        let direction = Direction::from_points2d(reference, Vec2::new(target.x, target.y));
        let prone = |hands| {
            (
                Gesture::body(BodyGesture::Prone(direction)).with_hands(hands),
                vec![],
            )
        };
        let total = Seconds(THROW_STEP_SECONDS / w.speed());

        let Some(grenade) = self.grenade_of(individual, nature) else {
            // Should not happen as such orders are skipped when there is no matching grenade
            return prone(HandsGesture::Idle);
        };
        let (position, grenade) = grenade;

        match individual.gesture.hands.clone() {
            HandsGesture::Idle => prone(HandsGesture::Reloading(U8Progress::zero())),
            HandsGesture::Reloading(progress) => {
                let (progress, _) = progress_(tick, progress, total);
                match progress.finished() {
                    true => prone(HandsGesture::Aiming(U8Progress::zero())),
                    false => prone(HandsGesture::Reloading(progress)),
                }
            }
            HandsGesture::Aiming(progress) => {
                let (progress, lag) = progress_(tick, progress, total);
                if !progress.finished() {
                    return prone(HandsGesture::Aiming(progress));
                }

                #[cfg(feature = "battle_instrument")]
                tracing::info!(name="battle-throw", i=self.i.0, side=?individual.side, nature=?nature);
                let updates = self.throw(individual, position, grenade, target, lag, direction);

                (
                    Gesture::body(BodyGesture::Prone(direction)).with_hands(HandsGesture::Idle),
                    updates,
                )
            }
        }
    }

    /// Grenade leaves the hands: consume it, schedule its explosion and finish the order.
    fn throw(
        &self,
        individual: &Individual,
        position: usize,
        grenade: &IndexedGrenade,
        target: WorldVec2,
        lag: Duration,
        direction: Direction,
    ) -> Updates {
        let mut weapons = individual.weapons.clone();
        weapons.grenades.remove(position);

        let landing = self.throw_landing(individual, target);
        let explosion = grenade.inner().clone().into();
        let origin = ExplosionOrigin::Grenade(grenade.index());
        let explode = runner::update::Update::SpawnExplosion(explosion, landing, origin);
        let at = Instant::now().checked_sub(lag).unwrap_or_else(Instant::now);

        let weapons = oc_individual::Update::SetWeapons(weapons);
        let mut updates = vec![
            runner::update::Update::UpdateIndividual(self.i, weapons),
            runner::update::Update::Schedule(at, Box::new(explode)),
        ];
        if let Some(sound) = self.world.mod_.pick_unpin_grenade_sound() {
            updates.push(runner::update::Update::PlaySoundWithGeo(
                sound,
                individual.position,
                individual.region(),
                oc_physics::fx::SoundBus::Effects,
            ));
        }
        // Throw decided without order (ex: enemy near) has no order to finish
        if matches!(
            individual.orders.first(),
            Some(Order::Grenade(_) | Order::Smoke(_))
        ) {
            updates.extend(self.accomplished_updates(direction));
        }
        updates
    }

    fn hands(
        &self,
        individual: &Individual,
        weapon: &oc_individual::Weapon,
        kind: WeaponKind,
        fire: impl FnOnce(&IndexedWeapon, AmmunitionIndex, Duration) -> Updates,
    ) -> (HandsGesture, Updates) {
        let w = &self.world.w;
        let tick = w.individual_tick();

        match (weapon.filled, individual.gesture.hands.clone()) {
            // The weapon is filled and ready to fire: all other than aiming imply to aim
            (Some(_), HandsGesture::Idle | HandsGesture::Reloading(_)) => {
                (HandsGesture::Aiming(U8Progress::zero()), vec![])
            }
            // If already aiming, continue
            (Some((_, ammunition)), HandsGesture::Aiming(progress)) => {
                let weapon = self.world.mod_.weapon(weapon.i);
                let total = weapon.aim(w) * individual.aim_factor();
                let (progress, lag) = progress_(tick, progress, total);
                tracing::trace!(name="individual-processor-hands-aiming", i=?self.i, progress=progress.0);

                match progress.finished() {
                    // If aiming is finished, fire
                    true => (HandsGesture::Idle, fire(weapon, ammunition, lag)),
                    false => (HandsGesture::Aiming(progress), vec![]),
                }
            }
            // The weapon is not ready to fire: all other than reloading imply start reloading
            (None, HandsGesture::Idle | HandsGesture::Aiming(_)) => {
                match self.can_reload(individual, weapon) {
                    true => (
                        HandsGesture::Reloading(U8Progress::zero()),
                        self.reload_sound(individual, weapon),
                    ),
                    // This weapon can't be reloaded, next individual tick will choose another weapon if any
                    false => (HandsGesture::Idle, vec![]),
                }
            }
            // If already reloading, continue
            (None, HandsGesture::Reloading(progress)) => {
                let weapon = self.world.mod_.weapon(weapon.i);
                let total = weapon.reload(w) * individual.reload_factor();
                let (progress, _) = progress_(tick, progress, total);
                tracing::trace!(name="individual-processor-hands-reloading", i=?self.i, progress=progress.0);

                match progress.finished() {
                    // If reloading is finished, update weapons and start to aim
                    // FIXME BS NOW: we must use reload exceedance to set the already progressed aiming
                    true => (
                        HandsGesture::Aiming(U8Progress::zero()),
                        self.reloaded(individual, kind),
                    ),
                    false => (HandsGesture::Reloading(progress), vec![]),
                }
            }
        }
    }
}

pub fn progress_(
    tick: oc_root::utils::Frequency,
    progress: U8Progress,
    total: Seconds, // FIXME BS NOW: Duration ? (replace Seconds by Duration ?)
) -> (U8Progress, Duration) {
    let total_ticks = total.0 / tick.period();
    let u8_tick = (255. / total_ticks).ceil() as u8;
    let (progress, exceedance) = progress.tick(u8_tick);

    // Ratio of lag to remove
    let exceedance = exceedance.0 as f32 / u8_tick as f32;
    let default_lag = tick.interval();
    let lag = default_lag - (default_lag.mul_f32(exceedance));

    (progress, lag)
}
