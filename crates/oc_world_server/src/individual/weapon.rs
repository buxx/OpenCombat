use std::time::Duration;

use glam::Vec3;
use oc_individual::{Individual, WeaponKind, order::Order};
use oc_mod::grenade::{GrenadeNature, IndexedGrenade};
use oc_projectile::spawn::SpawnProjectiles;
#[cfg(feature = "debug")]
use oc_root::WorldConfig;
use oc_root::{
    WcfgFrom,
    geo::{WorldVec2, WorldVec3},
    opacity::CumulatedOpacity,
    physics::Meters,
};
use oc_utils::{if_true, let_some, random::direction_with_inaccuracy_down};

use super::Processor;
use crate::runner;

/// Radius around the order position where a grenade lands, for a non-experienced individual
const THROW_SCATTER_METERS: Meters = Meters(5.0);
/// Radius added to scatter by 1.0 of opacity at the order position
const THROW_SCATTER_OPACITY_METERS: Meters = Meters(5.0);

impl Processor<'_> {
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

        let plus_z = individual.gesture.body.weapon_z().pixels(&self.world.w);
        let from = WorldVec3 {
            x: individual.position.x,
            y: individual.position.y,
            z: individual.position.z + plus_z,
        };

        let direction = (target - from).normalize_or_zero();
        let direction = Vec3::new(direction.x, direction.y, direction.z);
        let directions = (0..(repeat * shot.rounds()))
            .map(|_| {
                // FIXME BS NOW: inaccuracy could be different :
                //     - bad opacity should make random target, not increase inaccuracy
                #[cfg(feature = "debug")]
                let spread = match WorldConfig::inaccuracy_spread_enabled() {
                    true => WorldConfig::inaccuracy_spread(),
                    false => self.inaccuracy(individual, weapon, opacity),
                };
                #[cfg(not(feature = "debug"))]
                let spread = self.inaccuracy(individual, weapon, opacity);
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

    // TODO: introduce inaccuracy with recoil ?
    fn inaccuracy(
        &self,
        individual: &Individual,
        weapon: &oc_mod::weapons::IndexedWeapon,
        opacity: CumulatedOpacity,
    ) -> f32 {
        // TODO: Add time to aim factor ?
        let w = &self.world.w;
        let individual_xp_inaccuracy = individual.xp_inaccuracy(w);
        let individual_gesture_inaccuracy = individual.gesture.inaccuracy(w);
        let weapon_inaccuracy = weapon.inaccuracy(w);
        let individual_suppress_inaccuracy = individual.suppress_inaccuracy(w);
        let opacity_inaccuracy = opacity.inaccuracy(w);

        tracing::trace!(name="individual-processor-inaccuracy", i=?self.i,
            individual_xp_inaccuracy=individual_xp_inaccuracy,
            individual_gesture_inaccuracy=individual_gesture_inaccuracy,
            weapon_inaccuracy=weapon_inaccuracy,
            individual_suppress_inaccuracy=individual_suppress_inaccuracy,
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
