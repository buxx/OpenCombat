// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use derive_more::Constructor;
use getset::WithSetters;
use oc_geo::Geo;
use oc_geo::UpdateGeo;
use oc_geo::region::Region;
use oc_geo::region::RegionXy;
use oc_geo::region::WorldRegionIndex;
use oc_geo::tile::TileXy;
use oc_geo::tile::WorldTileIndex;
use oc_mod::Mod;
use oc_mod::ammunition::AmmunitionIndex;
use oc_mod::illustration::IllustrationIndex;
use oc_mod::magazine::MagazineIndex;
use oc_mod::nature::Traversability;
use oc_mod::vehicle::CrewRole;
use oc_mod::{grenade::GrenadeIndex, weapons::WeaponIndex};
use oc_physics::Force;
use oc_physics::IgnoreSide;
use oc_physics::Physic;
use oc_physics::UpdatePhysic;
use oc_physics::collision::Material;
use oc_physics::effect::Effect;
use oc_physics::volume::Volume;
use oc_root::WEAPON_Z_PRONE;
use oc_root::WEAPON_Z_STANDUP;
use oc_root::WcfgFrom;
use oc_root::WcfgInto;
use oc_root::WorldConfig;
use oc_root::behavior::{Stun, Suppress};
use oc_root::geo::WorldVec2;
use oc_root::geo::WorldVec3;
use oc_root::material::MaterialKind;
use oc_root::physics::Meters;
use oc_root::side::Side;
use oc_root::utils::U8Progress;
#[cfg(feature = "bevy")]
use oc_root::y::V;
use oc_utils::collections::WithIds;
use oc_utils::d2::Direction;
use oc_utils::d2::Xy;
use oc_vehicle::VehicleIndex;

use crate::behavior::Behavior;
use crate::behavior::Intent;
use crate::order::Order;

pub mod behavior;
pub mod network;
pub mod order;
pub mod squad;

pub const INDIVIDUAL_STAND_UP_VOLUME_WIDTH: Meters = Meters(0.8);
pub const INDIVIDUAL_STAND_UP_VOLUME_HEIGHT: Meters = Meters(0.8);
pub const INDIVIDUAL_STAND_UP_VOLUME_DEPTH: Meters = Meters(1.8);

// When maintain index of tile/individual, consider this shape to cover prone and rotation
pub const INDIVIDUAL_INDEXATION_SHAPE: Meters = Meters(2.0);

pub const INDIVIDUAL_PRONE_VOLUME_WIDTH: Meters = Meters(0.8);
pub const INDIVIDUAL_PRONE_VOLUME_HEIGHT: Meters = Meters(1.8);
pub const INDIVIDUAL_PRONE_VOLUME_DEPTH: Meters = Meters(0.8);

// Must stay under the navmesh agent radius (2.5px), or individuals cut corners and enter walls.
pub const WANT_STOP_TOLERANCE: f32 = 1.0;

oc_utils::index_type! {
    pub struct IndividualIndex(pub u64);
}

#[oc_derive::archivable]
#[derive(Debug, PartialEq, Constructor, Clone, WithSetters)]
#[allow(clippy::too_many_arguments)]
pub struct Individual {
    pub side: Side,
    #[getset(set_with = "pub")]
    pub illustration: IllustrationIndex,
    #[getset(set_with = "pub")]
    pub xp: Xp,
    pub position: WorldVec3,
    pub tile: WorldTileIndex,
    pub region: WorldRegionIndex,
    pub orders: Vec<Order>,
    /// Behavior is the general behavior of individual, like defending a direction
    pub behavior: Behavior,
    /// Forces are physical forces applied to this individual
    pub forces: Vec<Force>,
    /// Status is "in game" status, like be able to act, or not (dead, incapacitated, etc)
    pub status: Status,
    /// Gesture is the physical gesture of individual, like walking to take its defending position
    pub gesture: Gesture,
    /// Intent is the intention of individual, like taking cover, to accomplish its defending behavior
    pub intent: Intent,
    pub suppress: Suppress,
    pub weapons: Weapons,
    pub magazines: Vec<MagazineIndex>,
    /// Vehicle (and crew role in it) this individual is embarked in
    #[getset(set_with = "pub")]
    pub embarked: Option<(VehicleIndex, CrewRole)>,
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Update {
    SetPosition(WorldVec3),
    SetForces(Vec<Force>),
    SetOrders(Vec<Order>),
    SetBehavior(Behavior),
    SetGesture(Gesture),
    SetStatus(Status),
    SetIntent(Intent),
    Accomplished,
    MoveStepAccomplished,
    SetWeapons(Weapons),
    SetSuppress(Suppress),
    SetMagazines(Vec<MagazineIndex>),
}

impl Region for Individual {
    fn region(&self) -> WorldRegionIndex {
        self.region
    }

    fn set_region(&mut self, value: WorldRegionIndex) {
        self.region = value;
    }
}

impl From<i32> for IndividualIndex {
    fn from(value: i32) -> Self {
        Self(value as u64)
    }
}

impl Individual {
    /// Apply an update (as emitted by server) on this individual.
    pub fn apply(&mut self, w: &WorldConfig, update: &Update) -> Vec<Effect> {
        match update {
            Update::SetPosition(position) => return self.reposition(w, *position),
            Update::SetBehavior(behavior) => self.behavior = behavior.clone(),
            Update::SetOrders(orders) => self.orders = orders.clone(),
            Update::SetForces(forces) => self.forces = forces.clone(),
            Update::SetStatus(status) => self.status = *status,
            Update::SetGesture(gesture) => self.gesture = gesture.clone(),
            Update::SetIntent(intent) => self.intent = intent.clone(),
            Update::Accomplished => {
                if !self.orders.is_empty() {
                    self.orders.remove(0);
                }
            }
            Update::MoveStepAccomplished => self.intent.move_step_accomplished(),
            Update::SetWeapons(weapons) => self.weapons = weapons.clone(),
            Update::SetSuppress(suppress) => self.suppress = *suppress,
            Update::SetMagazines(magazines) => self.magazines = magazines.clone(),
        }

        vec![]
    }

    /// Set position, tile and region according to new `position`. Returns effects of this move.
    fn reposition(&mut self, w: &WorldConfig, position: WorldVec3) -> Vec<Effect> {
        let tile: TileXy = position.into_(w);
        let region: RegionXy = tile.into_(w);
        let tile: WorldTileIndex = tile.into_(w);
        let region: WorldRegionIndex = region.into_(w);

        let (before, region_before) = (self.position, self.region);
        self.position = position;
        self.set_tile(tile);
        self.set_region(region);

        let mut effects = vec![Effect::Position {
            before,
            after: position,
        }];
        if region_before != region {
            effects.push(Effect::Region {
                before: region_before,
                after: region,
            });
        }

        effects
    }

    pub fn fresh(w: &WorldConfig, side: Side, position: WorldVec3) -> Self {
        let tile_xy = TileXy(Xy(
            position.x as u64 / w.geo_pixels_per_tile(),
            position.y as u64 / w.geo_pixels_per_tile(),
        ));
        let tile = WorldTileIndex::from_(tile_xy, w);
        let region = WorldRegionIndex::from_(tile, w);
        Self::new(
            side,
            IllustrationIndex(0),
            Xp::AVERAGE,
            position,
            tile,
            region,
            vec![],
            Behavior::Idle(Direction::default()),
            vec![],
            Status::Operational,
            Gesture::body(BodyGesture::StandUp(Direction::default())),
            Intent::Idle(Direction::default()),
            Suppress::zero(),
            Weapons::default(),
            vec![],
            None,
        )
    }

    pub fn with_behavior(mut self, value: Behavior) -> Self {
        self.behavior = value;
        self
    }

    pub fn with_gesture(mut self, value: Gesture) -> Self {
        self.gesture = value;
        self
    }

    pub fn with_intent(mut self, value: Intent) -> Self {
        self.intent = value;
        self
    }

    pub fn with_weapons(mut self, value: Weapons) -> Self {
        self.weapons = value;
        self
    }

    pub fn with_magazines(mut self, value: Vec<MagazineIndex>) -> Self {
        self.magazines = value;
        self
    }

    pub fn tile(&self) -> WorldTileIndex {
        self.tile
    }

    pub fn region(&self) -> WorldRegionIndex {
        self.region
    }

    pub fn can_act(&self) -> bool {
        match self.status {
            Status::Operational => true,
            Status::Stunned(_) => false,
            Status::Dead => false,
        }
    }

    pub fn can_leadership(&self) -> bool {
        match self.status {
            Status::Operational | Status::Stunned(_) => true,
            Status::Dead => false,
        }
    }

    pub fn can_follow_orders(&self) -> bool {
        match self.status {
            Status::Operational | Status::Stunned(_) => true,
            Status::Dead => false,
        }
    }

    pub fn can_receive_proximity(&self) -> bool {
        match self.status {
            Status::Operational | Status::Stunned(_) => true,
            Status::Dead => false,
        }
    }

    pub fn can_lov(&self) -> bool {
        match self.status {
            Status::Operational | Status::Stunned(_) => true,
            Status::Dead => false,
        }
    }

    /// True if compute lov on it (when dead or incapacitated, false)
    pub fn is_lov_target(&self) -> bool {
        match self.status {
            Status::Operational | Status::Stunned(_) => true,
            Status::Dead => false,
        }
    }

    /// True if consider it as solid (when dead or incapacitated, false)
    pub fn is_solid(&self) -> bool {
        match self.status {
            Status::Operational | Status::Stunned(_) => true,
            Status::Dead => false,
        }
    }

    pub fn is_always_visible(&self) -> bool {
        match self.status {
            Status::Operational | Status::Stunned(_) => false,
            Status::Dead => true, // Dead are always displayed at screen
        }
    }

    pub fn xp_inaccuracy(&self, _w: &WorldConfig) -> f32 {
        // TODO
        0.0
    }

    pub fn suppress_inaccuracy(&self, w: &WorldConfig) -> f32 {
        self.suppress.normalize() * w.suppress_inaccuracy()
    }

    pub fn action_hud_label(&self) -> &str {
        match self.behavior {
            Behavior::Idle(_) => "Idle",
            Behavior::Walk(_) => "Walk",
            Behavior::Run(_) => "Run",
            Behavior::Crawl(_) => "Crawl",
            Behavior::Defend(_) => "Defend",
            Behavior::Hide(_) => "Hide",
            Behavior::Engage(_) => match self.gesture.hands {
                HandsGesture::Idle => "Engage",
                HandsGesture::Reloading(_) => "Reloading",
                HandsGesture::Aiming(_) => "Aiming",
            },
            Behavior::Suppress(_) => match self.gesture.hands {
                HandsGesture::Idle => "Suppress",
                HandsGesture::Reloading(_) => "Reloading",
                HandsGesture::Aiming(_) => "Aiming",
            },
            Behavior::Grenade(_) => match self.gesture.hands {
                HandsGesture::Idle => "Grenade",
                HandsGesture::Reloading(_) => "Reloading",
                HandsGesture::Aiming(_) => "Aiming",
            },
            Behavior::Smoke(_) => match self.gesture.hands {
                HandsGesture::Idle => "Smoke",
                HandsGesture::Reloading(_) => "Reloading",
                HandsGesture::Aiming(_) => "Aiming",
            },
        }
    }

    // FIXME: add other factors like suppress
    pub fn aim_factor(&self) -> f32 {
        1.0 + (1.0 - self.xp.f32())
    }

    // FIXME: add other factors like suppress
    pub fn reload_factor(&self) -> f32 {
        1.0 + (1.0 - self.xp.f32())
    }
}

impl Physic for Individual {
    fn position(&self, _: &WorldConfig) -> WorldVec3 {
        self.position
    }

    fn forces(&self, _: &WorldConfig) -> &Vec<Force> {
        &self.forces
    }

    fn volumes(
        &self,
        ref_: WorldVec3,
        _: bool,
        w: &WorldConfig,
        _mod_: &Mod,
    ) -> Vec<(Volume, Traversability, Direction)> {
        let direction = self.gesture.direction();
        let traversable = !self.is_solid();
        let cube = match self.gesture.body {
            BodyGesture::StandUp(_) | BodyGesture::Walking(_) | BodyGesture::Running(_) => {
                Volume::Cube {
                    x: 0.,
                    y: 0.,
                    z: 0.,
                    width: INDIVIDUAL_STAND_UP_VOLUME_WIDTH.pixels(w),
                    height: INDIVIDUAL_STAND_UP_VOLUME_HEIGHT.pixels(w),
                    depth: INDIVIDUAL_STAND_UP_VOLUME_DEPTH.pixels(w),
                }
                .with_ref(ref_)
            }
            BodyGesture::Crawling(_direction) | BodyGesture::Prone(_direction) => Volume::Cube {
                x: 0.,
                y: 0.,
                z: 0.,
                width: INDIVIDUAL_PRONE_VOLUME_WIDTH.pixels(w),
                height: INDIVIDUAL_PRONE_VOLUME_HEIGHT.pixels(w),
                depth: INDIVIDUAL_PRONE_VOLUME_DEPTH.pixels(w),
            }
            .with_ref(ref_),
        };
        vec![(
            cube,
            Traversability {
                individual: true, // TODO: prevent individual collisions ? Will need enhance physic model ...
                projectile: traversable,
            },
            direction,
        )]
    }

    fn ignore_side(&self) -> IgnoreSide {
        IgnoreSide::None
    }

    fn side(&self) -> Option<Side> {
        Some(self.side)
    }

    fn receive_proximity(&self) -> bool {
        self.can_receive_proximity()
    }

    fn ignore_underground(&self) -> bool {
        true
    }

    fn receive_z(&self) -> bool {
        true
    }

    fn want_stop(&self) -> Option<(WorldVec2, f32)> {
        self.intent
            .path()
            .and_then(|(_, p)| p.first())
            .map(|p| (*p, WANT_STOP_TOLERANCE))
    }
}

impl UpdatePhysic for Individual {
    fn set_position(&mut self, value: WorldVec3) {
        self.position = value;
    }

    fn push_force(&mut self, value: Force) {
        self.forces.push(value)
    }

    fn remove_force(&mut self, value: &Force) {
        self.forces.retain(|f| f != value)
    }

    fn set_volumes(&self, _value: Vec<(Volume, Traversability)>) {
        // No update volume of an individual (for now ...)
    }
}

impl Geo for Individual {
    fn tile(&self) -> WorldTileIndex {
        self.tile
    }
}

impl UpdateGeo for Individual {
    fn set_tile(&mut self, value: WorldTileIndex) {
        self.tile = value;
    }
}

impl Material for Individual {
    fn kind(&self) -> Option<MaterialKind> {
        Some(MaterialKind::Individual)
    }
}

impl<'a> WithIds<IndividualIndex, &'a Individual> for &'a [Individual] {
    fn with_ids(&self) -> Vec<(IndividualIndex, &'a Individual)> {
        self.iter()
            .enumerate()
            .map(|(i, individual)| (i.into(), individual))
            .collect()
    }
}

#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Operational,
    // FIXME: will lost/not apply real status
    Stunned(Stun),
    Dead,
}

impl Status {
    pub fn can_step(&self) -> bool {
        match self {
            Status::Operational | Status::Stunned(_) => true,
            Status::Dead => false,
        }
    }

    pub fn is_target(&self) -> bool {
        match self {
            Status::Operational | Status::Stunned(_) => true,
            Status::Dead => false,
        }
    }

    pub fn hud_label(&self) -> &str {
        match self {
            Status::Operational => "Operational",
            Status::Stunned(_) => "Stunned",
            Status::Dead => "Dead",
        }
    }

    pub fn is_dead(&self) -> bool {
        matches!(self, Self::Dead)
    }
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub struct Gesture {
    pub body: BodyGesture,
    pub hands: HandsGesture,
}

impl Gesture {
    pub fn body(gesture: BodyGesture) -> Self {
        Self {
            body: gesture,
            hands: HandsGesture::Idle,
        }
    }

    pub fn with_hands(mut self, value: HandsGesture) -> Self {
        self.hands = value;
        self
    }

    pub fn inaccuracy(&self, w: &WorldConfig) -> f32 {
        match self.body {
            BodyGesture::StandUp(_) => w.standup_inaccuracy(),
            BodyGesture::Walking(_) => w.walking_inaccuracy(),
            BodyGesture::Running(_) => w.running_inaccuracy(),
            BodyGesture::Crawling(_) => w.crawling_inaccuracy(),
            BodyGesture::Prone(_) => w.prone_inaccuracy(),
        }
    }
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum BodyGesture {
    StandUp(Direction),
    Walking(Direction),
    Running(Direction),
    Crawling(Direction),
    Prone(Direction),
}

impl BodyGesture {
    /// Meters (from ground) to consider as weapon height
    pub fn weapon_z(&self) -> Meters {
        match self {
            BodyGesture::StandUp(_) | BodyGesture::Walking(_) | BodyGesture::Running(_) => {
                WEAPON_Z_STANDUP
            }
            BodyGesture::Crawling(_) | BodyGesture::Prone(_) => WEAPON_Z_PRONE,
        }
    }

    /// Meters (from ground) to consider as target point
    pub fn target_z(&self) -> Meters {
        match self {
            BodyGesture::StandUp(_) | BodyGesture::Walking(_) | BodyGesture::Running(_) => {
                Meters(0.75)
            }
            BodyGesture::Crawling(_) | BodyGesture::Prone(_) => Meters(0.30),
        }
    }
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum HandsGesture {
    Idle,
    Reloading(U8Progress),
    Aiming(U8Progress),
}

impl Gesture {
    #[cfg(feature = "bevy")]
    pub fn rotation(&self, v: V) -> bevy::prelude::Quat {
        let angle = self.direction().angle(v);
        bevy::prelude::Quat::from_rotation_z(angle.0)
    }

    pub fn direction(&self) -> Direction {
        match self.body {
            BodyGesture::StandUp(direction)
            | BodyGesture::Walking(direction)
            | BodyGesture::Running(direction)
            | BodyGesture::Crawling(direction)
            | BodyGesture::Prone(direction) => direction,
        }
    }
}

#[oc_derive::archivable]
#[derive(Debug, PartialEq, Constructor, Clone, Default)]
pub struct Weapons {
    pub primary: Option<Weapon>,
    pub grenades: Vec<GrenadeIndex>,
}

impl Weapons {
    pub fn get_mut(&mut self, kind: &WeaponKind) -> Option<&mut Weapon> {
        match kind {
            WeaponKind::Primary => self.primary.as_mut(),
        }
    }
}

#[oc_derive::archivable]
#[derive(Debug, PartialEq, Constructor, Clone)]
pub struct Weapon {
    pub i: WeaponIndex,
    pub filled: Option<(MagazineIndex, AmmunitionIndex)>, // TODO: probably rethink arch for weapon without magazine
    pub filled_count: u16,
}

#[oc_derive::archivable]
#[derive(Debug, PartialEq, Clone)]
pub enum WeaponKind {
    Primary,
}

#[oc_derive::archivable]
#[derive(Debug, PartialEq, Clone)]
pub struct Xp(pub u8);

impl Xp {
    pub const AVERAGE: Xp = Xp(128);

    pub fn f32(&self) -> f32 {
        self.0 as f32 / 255.
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn individual_with_xp(xp: u8) -> Individual {
        let w = WorldConfig::new(1000, 1000);
        Individual::fresh(&w, Side::A, WorldVec3::new(10., 10., 0.)).with_xp(Xp(xp))
    }

    #[test]
    fn aim_factor_is_two_without_xp() {
        assert_eq!(individual_with_xp(0).aim_factor(), 2.0);
    }

    #[test]
    fn aim_factor_is_one_with_max_xp() {
        assert_eq!(individual_with_xp(255).aim_factor(), 1.0);
    }

    #[test]
    fn aim_factor_average_xp_is_about_one_and_a_half() {
        let factor = individual_with_xp(Xp::AVERAGE.0).aim_factor();
        assert!((factor - 1.5).abs() < 0.01, "got {factor}");
    }

    #[test]
    fn aim_factor_decreases_as_xp_increases() {
        assert!(individual_with_xp(200).aim_factor() < individual_with_xp(50).aim_factor());
    }
}
