use std::time::Duration;

use glam::Vec3;
use oc_individual::{Individual, WeaponKind, order::Order};
use oc_mod::grenade::{GrenadeNature, IndexedGrenade};
use oc_physics::Force;
use oc_projectile::spawn::SpawnProjectiles;
#[cfg(feature = "debug")]
use oc_root::WorldConfig;
use oc_root::{
    WcfgFrom,
    geo::{WorldVec2, WorldVec3},
    opacity::CumulatedOpacity,
    physics::{Meters, MetersSeconds},
};
use oc_utils::{if_true, let_some, random::direction_with_inaccuracy_down};

use super::Processor;
use crate::runner;

/// Radius around the order position where a grenade lands, for a non-experienced individual
const THROW_SCATTER_METERS: Meters = Meters(5.0);
/// Radius added to scatter by 1.0 of opacity at the order position
const THROW_SCATTER_OPACITY_METERS: Meters = Meters(5.0);

impl Processor<'_> {
    /// Target velocity
    pub fn velocity(&self, target: &Individual) -> Vec3 {
        let forces = match target.embarked {
            Some((vehicle, _)) => &self.world.vehicle(vehicle).forces,
            None => &target.forces,
        };
        let pixels_per_meters = self.world.w.geo_pixels_per_meters();
        forces
            .iter()
            .fold(Vec3::ZERO, |velocity, force| match force {
                Force::Translation(direction, speed) => {
                    velocity + Vec3::from(*direction) * speed.0 * pixels_per_meters
                }
            })
    }

    /// Position where `shooter` aims to hit the moving `target`.
    pub fn anticipate(
        &self,
        shooter: &Individual,
        target: &Individual,
        from: WorldVec3,
        aim: WorldVec3,
        speed: MetersSeconds,
        lag: Duration,
    ) -> WorldVec3 {
        let velocity = self.velocity(target);
        if velocity == Vec3::ZERO {
            return aim;
        }

        let w = &self.world.w;
        let error = shooter.anticipation_error();
        let factor = 1.0 + (fastrand::f32() * 2.0 - 1.0) * error;
        let speed = speed.0 * w.geo_pixels_per_meters();
        let lag = lag.as_secs_f32() * w.speed();

        anticipated(from, aim, velocity, speed, lag, factor)
    }

    /// Random position around the order position where the grenade lands. Scatter is reduced by
    /// experience (divided by 1 up to 10 for max experience) and increased by order position opacity.
    pub fn throw_landing(&self, individual: &Individual, target: WorldVec2) -> WorldVec2 {
        let w = &self.world.w;
        let mod_ = self.world.mod_();
        let xy = oc_geo::tile::TileXy::from_([target.x, target.y], w);
        let index = oc_geo::tile::WorldTileIndex::from_(xy, w);
        let opacity = self
            .world
            .tile(index)
            .map(|t| mod_.nature(t.nature).opacity)
            .unwrap_or(0.0);

        let divisor = (individual.xp.f32() * 10.0).max(1.0);
        let rayon = (THROW_SCATTER_METERS * (1.0 / divisor)).pixels(w)
            + (THROW_SCATTER_OPACITY_METERS * opacity).pixels(w);

        let distance = fastrand::f32().sqrt() * rayon;
        let angle = fastrand::f32() * std::f32::consts::TAU;
        WorldVec2::new(
            target.x + angle.cos() * distance,
            target.y + angle.sin() * distance,
        )
    }

    pub fn grenade_of(
        &self,
        individual: &Individual,
        nature: GrenadeNature,
    ) -> Option<(usize, &IndexedGrenade)> {
        individual
            .weapons
            .grenades
            .iter()
            .enumerate()
            .map(|(position, grenade)| (position, self.world.mod_.grenade(*grenade)))
            .find(|(_, grenade)| grenade.nature == nature)
    }

    pub fn matching_grenade(
        &self,
        individual: &Individual,
        order: &Order,
    ) -> Option<(usize, &IndexedGrenade)> {
        let nature = match order {
            Order::Grenade(_) => GrenadeNature::Frag,
            Order::Smoke(_) => GrenadeNature::Smoke,
            _ => return None,
        };
        self.grenade_of(individual, nature)
    }

    fn compatible_magazine<'m>(
        &self,
        individual: &Individual,
        weapon: &'m oc_mod::weapons::Weapon,
    ) -> Option<(usize, &'m oc_mod::magazine::IndexedMagazine)> {
        let compatibles = weapon.magazines();

        individual
            .magazines
            .iter()
            .enumerate()
            .find_map(|(position, carried)| {
                let magazine = compatibles.iter().find(|m| m.index() == *carried)?;
                Some((position, magazine))
            })
    }

    // FIXME: reload is different according to weapon type. For now, its simple version where reload
    // from filled weapon or magazine.
    pub fn can_reload(&self, individual: &Individual, weapon: &oc_individual::Weapon) -> bool {
        if_true!(weapon.filled.is_some(), return true);
        let weapon = self.world.mod_.weapon(weapon.i);
        self.compatible_magazine(individual, weapon).is_some()
    }

    pub fn reload_sound(
        &self,
        individual: &Individual,
        weapon: &oc_individual::Weapon,
    ) -> Vec<runner::update::Update> {
        let weapon = self.world.mod_.weapon(weapon.i);

        match weapon.reload_sound() {
            Some(sound) => vec![runner::update::Update::PlaySoundWithGeo(
                sound,
                individual.position,
                individual.region(),
                oc_physics::fx::SoundBus::Effects,
            )],
            None => vec![],
        }
    }

    pub fn reloaded(
        &self,
        individual: &Individual,
        kind: WeaponKind,
    ) -> Vec<runner::update::Update> {
        let mut weapons = individual.weapons.clone();
        let_some!(weapon = weapons.get_mut(&kind), return vec![]);
        let weapon_type = self.world.mod_.weapon(weapon.i);

        // FIXME: reload is different according to weapon type. For now, its simple version where reload
        // from filled weapon or magazine.
        if weapon.filled.is_some() {
            return vec![];
        }

        if let (Some((position, magazine)), Some(ammunition)) = (
            self.compatible_magazine(individual, weapon_type),
            weapon_type.ammunitions().first(),
        ) {
            let mut magazines = individual.magazines.clone();
            magazines.remove(position);
            weapon.filled = Some((magazine.index(), ammunition.index()));
            weapon.filled_count = magazine.capacity();

            return vec![
                runner::update::Update::UpdateIndividual(
                    self.i,
                    oc_individual::Update::SetWeapons(weapons),
                ),
                runner::update::Update::UpdateIndividual(
                    self.i,
                    oc_individual::Update::SetMagazines(magazines),
                ),
            ];
        }

        vec![]
    }

    #[allow(clippy::too_many_arguments)]
    pub fn spawn_projectile(
        &self,
        individual: &Individual,
        weapon: &oc_mod::weapons::IndexedWeapon,
        ammunition: oc_mod::ammunition::AmmunitionIndex,
        kind: WeaponKind,
        target: WorldVec3,
        opacity: CumulatedOpacity,
        lag: Duration,
    ) -> Vec<runner::update::Update> {
        // FIXME BS NOW: how choose mode ?
        let_some!(shot = weapon.shots().first(), return vec![]);
        let repeat = 1;

        let mut weapons = individual.weapons.clone();
        if let Some(weapon) = weapons.get_mut(&kind) {
            weapon.filled_count = weapon.filled_count.saturating_sub(1);
            if weapon.filled_count == 0 {
                weapon.filled = None;
            }
        }

        let from = self.weapon_position(individual);

        let distance = Meters::from_pixels(from.distance(target), &self.world.w);
        let direction = (target - from).normalize_or_zero();
        let direction = Vec3::new(direction.x, direction.y, direction.z);
        let directions = (0..(repeat * shot.rounds()))
            .map(|_| {
                // FIXME BS NOW: inaccuracy could be different :
                //     - bad opacity should make random target, not increase inaccuracy
                #[cfg(feature = "debug")]
                let spread = match WorldConfig::inaccuracy_spread_enabled() {
                    true => WorldConfig::inaccuracy_spread(),
                    false => self.inaccuracy(individual, weapon, opacity, distance),
                };
                #[cfg(not(feature = "debug"))]
                let spread = self.inaccuracy(individual, weapon, opacity, distance);
                let direction = direction_with_inaccuracy_down(
                    direction,
                    spread,
                    self.world.w.inaccuracy_down_factor(),
                );
                WorldVec3::new(direction.x, direction.y, direction.z)
            })
            .collect::<Vec<_>>();

        let spawn = SpawnProjectiles {
            weapon: weapon.index(),
            ammunition,
            shot: shot.index(),
            repeat: repeat as u8,
            from,
            // FIXME BS NOW: berk, create Direction3d
            directions,
            side: individual.side,
            lag,
            shooter: Some(self.i),
        };
        tracing::trace!(name="individual-processor-spawn-projectiles", i=?self.i, spawn=?spawn, weapons=?weapons);

        vec![
            runner::update::Update::SpawnProjectiles(spawn),
            runner::update::Update::UpdateIndividual(
                self.i,
                oc_individual::Update::SetWeapons(weapons),
            ),
        ]
    }

    /// Position from where the individual weapon fires
    pub fn weapon_position(&self, individual: &Individual) -> WorldVec3 {
        let plus_z = individual.gesture.body.weapon_z().pixels(&self.world.w);
        WorldVec3 {
            x: individual.position.x,
            y: individual.position.y,
            z: individual.position.z + plus_z,
        }
    }

    // TODO: introduce inaccuracy with recoil ?
    fn inaccuracy(
        &self,
        individual: &Individual,
        weapon: &oc_mod::weapons::IndexedWeapon,
        opacity: CumulatedOpacity,
        distance: Meters,
    ) -> f32 {
        // TODO: Add time to aim factor ?
        let w = &self.world.w;
        let individual_xp_inaccuracy = individual.xp_inaccuracy(w);
        let individual_gesture_inaccuracy = individual.gesture.inaccuracy(w);
        let weapon_inaccuracy = weapon.inaccuracy(w);
        let individual_suppress_inaccuracy = individual.suppress_inaccuracy(w);
        let opacity_inaccuracy = opacity.inaccuracy(w, distance);

        tracing::trace!(name="individual-processor-inaccuracy", i=?self.i,
            individual_xp_inaccuracy=individual_xp_inaccuracy,
            individual_gesture_inaccuracy=individual_gesture_inaccuracy,
            weapon_inaccuracy=weapon_inaccuracy,
            individual_suppress_inaccuracy=individual_suppress_inaccuracy,
            opacity=opacity.0,
            distance=distance.0,
            opacity_inaccuracy=opacity_inaccuracy,
        );

        w.base_inaccuracy()
            + individual_xp_inaccuracy
            + individual_gesture_inaccuracy
            + weapon_inaccuracy
            + individual_suppress_inaccuracy
            + opacity_inaccuracy
    }

    pub fn weapon<'w>(
        &self,
        individual: &'w Individual,
        _target: &'w Individual,
    ) -> Option<(WeaponKind, &'w oc_individual::Weapon)> {
        // TODO: several things can change (knife when contact, secondary weapon when no more bullet on primary, etc)
        // TODO: choose of this weapon must imply it can be reloaded, else, choose another
        individual
            .weapons
            .primary
            .as_ref()
            .map(|w| (WeaponKind::Primary, w))
    }

    pub fn primary_weapon<'w>(
        &self,
        individual: &'w Individual,
        _target: WorldVec2,
    ) -> Option<&'w oc_individual::Weapon> {
        individual.weapons.primary.as_ref()
    }
}

/// Position of a target (currently at `aim`, moving at `velocity`) when a projectile fired from
/// `from` at `speed` after `lag` (world seconds) reaches it; displacement multiplied by `factor`
/// (anticipation error).
fn anticipated(
    from: WorldVec3,
    aim: WorldVec3,
    velocity: Vec3,
    speed: f32,
    lag: f32,
    factor: f32,
) -> WorldVec3 {
    if speed <= 0.0 {
        return aim;
    }
    let aim_ = Vec3::from(aim);
    let from_ = Vec3::from(from);

    // Projectile flight time depends on the anticipated position: refine it a few times
    let mut target = aim_;
    for _ in 0..3 {
        let time = lag + from_.distance(target) / speed;
        target = aim_ + velocity * time;
    }
    let anticipated = aim_ + (target - aim_) * factor;

    WorldVec3::new(anticipated.x, anticipated.y, anticipated.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anticipated_static_target_is_target() {
        let from = WorldVec3::new(0., 0., 0.);
        let aim = WorldVec3::new(100., 0., 0.);
        assert_eq!(anticipated(from, aim, Vec3::ZERO, 50., 0., 1.), aim);
    }

    #[test]
    fn anticipated_moving_target_leads_by_flight_time() {
        let from = WorldVec3::new(0., 0., 0.);
        let aim = WorldVec3::new(100., 0., 0.);
        // Flight is 1 second (100px at 100px/s), target moves 10px/s on y
        let result = anticipated(from, aim, Vec3::new(0., 10., 0.), 100., 0., 1.);
        assert_eq!(result.x, 100.);
        // Refined flight time is a bit longer than 1 second as target moves away
        assert!(result.y > 10. && result.y < 10.1, "got {}", result.y);
    }

    #[test]
    fn anticipated_includes_lag() {
        let from = WorldVec3::new(0., 0., 0.);
        let aim = WorldVec3::new(100., 0., 0.);
        let result = anticipated(from, aim, Vec3::new(-10., 0., 0.), 1000., 1.0, 1.);
        // ~1 second lag + ~0.09 second flight
        assert!(result.x < 90. && result.x > 88., "got {}", result.x);
    }

    #[test]
    fn anticipated_displacement_scaled_by_error_factor() {
        let from = WorldVec3::new(0., 0., 0.);
        let aim = WorldVec3::new(100., 0., 0.);
        let velocity = Vec3::new(0., 10., 0.);
        let exact = anticipated(from, aim, velocity, 100., 0., 1.);
        let over = anticipated(from, aim, velocity, 100., 0., 2.);
        let none = anticipated(from, aim, velocity, 100., 0., 0.);
        assert!((over.y - exact.y * 2.).abs() < 0.001);
        assert_eq!(none, aim);
    }
}
