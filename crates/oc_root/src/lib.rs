// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

#[cfg(feature = "debug")]
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

#[cfg(feature = "bevy")]
use bevy::prelude::*;

use getset::{CopyGetters, Getters, WithSetters};

use crate::{
    behavior::{Stun, Suppress},
    opacity::CumulatedOpacity,
    physics::{Meters, MetersSeconds},
    utils::Frequency,
};

pub mod alter;
pub mod battle;
pub mod behavior;
pub mod craters;
pub mod end;
pub mod explosion;
pub mod files;
pub mod format;
pub mod geo;
pub mod identity;
pub mod ids;
pub mod material;
pub mod opacity;
pub mod physics;
pub mod side;
pub mod sprites;
pub mod static_;
pub mod utils;
pub mod uuids;
pub mod y;

pub const HUD_MINIMAP_SIZE: u32 = 400;
#[cfg(feature = "debug")]
static INACCURACY_SPREAD_RAW: AtomicU32 = AtomicU32::new(0);
#[cfg(feature = "debug")]
static INACCURACY_SPREAD_ENABLED: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "debug")]
static IMMORTALITY: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "debug")]
static SPEED_RAW: AtomicU32 = AtomicU32::new(10); // Will be divided by ten

pub const DEFAULT_FIGHT_DURATION: Duration = Duration::from_secs(15 * 60);
pub const DEFAULT_INTERIOR_PENETRATION_PROTECT: f32 = 0.2;
pub const DEFAULT_INTERIOR_DESTRUCTION_RESISTANCE: f32 = 0.15;
pub const DEFAULT_DECOR_DESTRUCTION_RESISTANCE: f32 = 0.5;

/// Salts given to `WorldTileIndex::pick`/`seeded` so each feature gets an uncorrelated
/// pseudo-random value for the same tile
pub const TILE_SALT_ROTATION: u64 = 0;
pub const TILE_SALT_ALTER_SPRITE: u64 = 1;
pub const TILE_SALT_INTERIOR_DESTRUCTION: u64 = 2;

/// Name of the official default mod
pub const DEFAULT_MOD: &str = "std1";

pub const SPAWN_ZONE_NONE_OPACITY: f32 = 0.75;
pub const SPAWN_ZONE_ENEMY_OPACITY: f32 = 0.95;

pub const WEAPON_Z_STANDUP: Meters = Meters(1.5);
pub const WEAPON_Z_PRONE: Meters = Meters(0.70);

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq, WithSetters, Getters, CopyGetters)]
pub struct WorldConfig {
    #[getset(get_copy = "pub")]
    world_width: u64,
    #[getset(get_copy = "pub")]
    world_height: u64,
    #[getset(get_copy = "pub")]
    region_width: u64,
    #[getset(get_copy = "pub")]
    region_height: u64,
    #[getset(get_copy = "pub")]
    tiles_count: u64,
    #[getset(get_copy = "pub")]
    regions_count: u64,
    #[getset(get_copy = "pub")]
    regions_width: u64,
    #[getset(get_copy = "pub")]
    regions_height: u64,
    #[getset(get_copy = "pub")]
    world_width_pixels: u64,
    #[getset(get_copy = "pub")]
    world_height_pixels: u64,
    #[getset(get_copy = "pub")]
    region_width_pixels: u64,
    #[getset(get_copy = "pub")]
    region_height_pixels: u64,
    #[getset(set_with = "pub")]
    speed: f32,
    #[getset(set_with = "pub")]
    individual_tick: Frequency,
    #[getset(set_with = "pub")]
    visibilities_tick: Frequency,
    #[getset(set_with = "pub")]
    squad_tick: Frequency,
    #[getset(set_with = "pub")]
    physics_tick: Frequency,
    #[getset(set_with = "pub")]
    scheduler_tick: Frequency,
    #[getset(get_copy = "pub", set_with = "pub")]
    interiors_tick: Frequency, // TODO: Only for GUI, maybe move it in gui
    #[getset(get_copy = "pub", set_with = "pub")]
    geo_pixels_per_meters: f32,
    #[getset(get_copy = "pub", set_with = "pub")]
    geo_pixels_per_tile: u64,
    #[getset(get_copy = "pub", set_with = "pub")]
    geo_meters_per_z: Meters,
    #[getset(get_copy = "pub", set_with = "pub")]
    minimap_width_pixels: u64,
    #[getset(get_copy = "pub", set_with = "pub")]
    minimap_height_pixels: u64,
    #[getset(get_copy = "pub", set_with = "pub")]
    formation_tiles_between_positions: u64,
    /// Tile radius around a formation position where deployment looks for the best opacity tile
    #[getset(get_copy = "pub", set_with = "pub")]
    formation_tiles_radius_placement: u64,
    #[getset(get_copy = "pub", set_with = "pub")]
    individual_visibility_until: CumulatedOpacity,
    /// Opacity removed when individual shoot since last visibility test
    #[getset(get_copy = "pub", set_with = "pub")]
    shoot_visibility_bonus: CumulatedOpacity,
    /// Inaccuracy start value
    #[getset(get_copy = "pub", set_with = "pub")]
    base_inaccuracy: f32,
    /// Inaccuracy value for 100% suppressed (50% suppressed will by 50% of this value)
    #[getset(get_copy = "pub", set_with = "pub")]
    suppress_inaccuracy: f32,
    #[getset(get_copy = "pub", set_with = "pub")]
    standup_inaccuracy: f32,
    #[getset(get_copy = "pub", set_with = "pub")]
    walking_inaccuracy: f32,
    #[getset(get_copy = "pub", set_with = "pub")]
    running_inaccuracy: f32,
    #[getset(get_copy = "pub", set_with = "pub")]
    crawling_inaccuracy: f32,
    #[getset(get_copy = "pub", set_with = "pub")]
    prone_inaccuracy: f32,
    /// Inaccuracy value of a vehicle gun fired while the vehicle is moving
    #[getset(get_copy = "pub", set_with = "pub")]
    vehicle_moving_inaccuracy: f32,
    /// Inaccuracy value for 100% opacity (50% opacity will by 50% of this value)
    #[getset(get_copy = "pub", set_with = "pub")]
    opacity_inaccuracy: f32,
    /// Fire distance at which opacity inaccuracy is exactly [`Self::opacity_inaccuracy`]. Opacity
    /// inaccuracy is scaled by `distance / reference` (bounded by min/max factors below), so bad
    /// visibility matters more on far targets.
    #[getset(get_copy = "pub", set_with = "pub")]
    opacity_inaccuracy_reference_distance: Meters,
    /// Minimum distance factor applied to opacity inaccuracy (close targets)
    #[getset(get_copy = "pub", set_with = "pub")]
    opacity_inaccuracy_min_distance_factor: f32,
    /// Maximum distance factor applied to opacity inaccuracy (far targets)
    #[getset(get_copy = "pub", set_with = "pub")]
    opacity_inaccuracy_max_distance_factor: f32,
    /// Factor applied to the downward part of the shot deviation (0.0..=1.0)
    #[getset(get_copy = "pub", set_with = "pub")]
    inaccuracy_down_factor: f32,
    /// To prevent problems due to "square 3d" (when gunner is close to the edge)
    #[getset(get_copy = "pub", set_with = "pub")]
    ignore_firsts_lov_tiles: u8,
    /// To prevent problems due to "square 3d" (when gunner is close to the edge)
    #[getset(get_copy = "pub", set_with = "pub")]
    ignore_firsts_physics_pixels: u8,
    /// Consider individual proximity in this tile rayon
    #[getset(get_copy = "pub", set_with = "pub")]
    proximity_individual_rayon: u64,
    /// When projectile near individual, increase individual suppress by this value at tick
    /// A proportional value of given distance will be used.
    #[getset(get_copy = "pub", set_with = "pub")]
    proximity_projectile_fly_tick_increase_value: (Meters, Suppress),
    /// When projectile collision near individual, increase individual suppress by this value at tick
    /// A proportional value of given distance will be used.
    #[getset(get_copy = "pub", set_with = "pub")]
    proximity_projectile_impact_tick_increase_value: (Meters, Suppress),
    /// Chance (0.0..=1.0) that a projectile impact plays a sound
    #[getset(get_copy = "pub", set_with = "pub")]
    impact_sound_chance: f32,
    /// Decrease each individual suppress value at tick by this value
    #[getset(get_copy = "pub", set_with = "pub")]
    individual_tick_decrease_suppress: Suppress,
    /// Decrease each stunned individual stun value at tick by this value
    #[getset(get_copy = "pub", set_with = "pub")]
    individual_tick_decrease_stunned: Stun,
    /// Stun value given to individuals caught by explosion stunned rayon
    #[getset(get_copy = "pub", set_with = "pub")]
    explosion_stunned_value: Stun,
    /// Stun value given to vehicle crew when a shell explodes on the vehicle chassis without
    /// penetrating it (30 seconds: decreased by `individual_tick_decrease_stunned` each second)
    #[getset(get_copy = "pub", set_with = "pub")]
    vehicle_crew_stunned_value: Stun,
    /// Individual with this or more suppress will hide (instead of make other things)
    #[getset(get_copy = "pub")]
    individual_suppress_limit_hide: Suppress,
    /// Distance where hiding individual will not engage
    #[getset(get_copy = "pub", set_with = "pub")]
    // TODO: When click hide order, permit use key (like Shift) to specify distance
    individual_hide_engage_distance: Meters,
    /// Distance under which an individual owning a frag grenade throws it on a visible enemy
    #[getset(get_copy = "pub", set_with = "pub")]
    individual_grenade_enemy_distance: Meters,
    #[getset(get_copy = "pub", set_with = "pub")]
    /// Distance where hiding opacity bonus is no more effective
    hide_opacity_bonus_ends: Meters,
    /// Hiding bonus factor for each meters between hided and watcher
    /// Computing is : hiding_opacity_bonus (or crawling_opacity_bonus) * hide_opacity_bonus_meter_factor
    /// for each meters.
    #[getset(get_copy = "pub", set_with = "pub")]
    hide_opacity_bonus_meter_factor: f32,
    /// Tile distance to take into account to determine best tile opacity (and hide_opacity_bonus)
    #[getset(get_copy = "pub", set_with = "pub")]
    tiles_radius_opacity_picking: u64,
    #[getset(get_copy = "pub", set_with = "pub")]
    walk_nominal_speed: MetersSeconds,
    #[getset(get_copy = "pub", set_with = "pub")]
    run_nominal_speed: MetersSeconds,
    #[getset(get_copy = "pub", set_with = "pub")]
    crawl_nominal_speed: MetersSeconds,
    #[getset(get_copy = "pub", set_with = "pub")]
    fight_duration: Duration,
    /// Side morale (percent) under which the battle end
    #[getset(get_copy = "pub", set_with = "pub")]
    morale_end_threshold: f32,
}

impl WorldConfig {
    pub fn new(world_width: u64, world_height: u64) -> Self {
        let region_width = 1000.min(world_width);
        let region_height = 1000.min(world_height);

        let speed = 1.0;

        #[cfg(feature = "debug")]
        if let Ok(speed) = std::env::var("OC_SPEED")
            && let Ok(speed) = speed.parse::<f32>()
        {
            SPEED_RAW.store((speed * 10.) as u32, Ordering::Relaxed);
        }

        let individual_tick = Frequency::each(Duration::from_secs(1));
        let visibilities_tick = Frequency::each(Duration::from_secs(2));
        let squad_tick = Frequency::each(Duration::from_secs(5));
        let physics_tick = Frequency::each(Duration::from_millis(500));
        let scheduler_tick = Frequency::each(Duration::from_millis(5));
        let interiors_tick = Frequency::each(Duration::from_secs(2));
        let geo_pixels_per_meters: f32 = 5.;
        let geo_pixels_per_tile: u64 = geo_pixels_per_meters as u64;

        let tiles_count = world_width * world_height;
        let regions_count = tiles_count / (region_width * region_height);
        let regions_width = world_width / region_width;
        let regions_height = world_height / region_height;
        let world_width_pixels = world_width * geo_pixels_per_tile;
        let world_height_pixels = world_height * geo_pixels_per_tile;
        let region_width_pixels = region_width * geo_pixels_per_tile;
        let region_height_pixels = region_height * geo_pixels_per_tile;
        let minimap_width_pixels: u64 = 2048;
        let minimap_height_pixels: u64 = 2048;
        let geo_meters_per_z = Meters(0.1);

        let formation_tiles_between_positions = 2;
        let formation_tiles_radius_placement = 2;
        let individual_visibility_until = CumulatedOpacity(0.6);
        let shoot_visibility_bonus = CumulatedOpacity(0.3);
        let base_inaccuracy = 0.05;
        let suppress_inaccuracy = 2.0;
        let standup_inaccuracy = 0.1;
        let walking_inaccuracy = 1.5;
        let running_inaccuracy = 2.5;
        let crawling_inaccuracy = 1.8;
        let prone_inaccuracy = 0.0;
        let vehicle_moving_inaccuracy = 3.0;
        let opacity_inaccuracy = 2.0;
        let opacity_inaccuracy_reference_distance = Meters(100.);
        let opacity_inaccuracy_min_distance_factor = 0.25;
        let opacity_inaccuracy_max_distance_factor = 3.0;
        let inaccuracy_down_factor = 0.25;
        let ignore_firsts_lov_tiles = 2;
        let ignore_firsts_physics_pixels = (geo_pixels_per_tile * 13).clamp(0, 255) as u8;

        let proximity_individual_rayon = 4;
        let proximity_projectile_fly_tick_increase_value = (Meters(20.), Suppress(50));
        let proximity_projectile_impact_tick_increase_value = (Meters(30.), Suppress(100));
        let impact_sound_chance = 0.35;
        let individual_tick_decrease_suppress = Suppress(20);
        let individual_tick_decrease_stunned = Stun(5);
        let explosion_stunned_value = Stun(255);
        let vehicle_crew_stunned_value = Stun(150);
        let individual_suppress_limit_hide = Suppress(200);
        let individual_hide_engage_distance = Meters(40.0);
        let individual_grenade_enemy_distance = Meters(15.0);
        let hide_opacity_bonus_ends = Meters(10.);
        let hide_opacity_bonus_meter_factor = 0.005;
        let tiles_radius_opacity_picking = 1;

        let walk_nominal_speed = MetersSeconds(1.2);
        let run_nominal_speed = MetersSeconds(2.7);
        let crawl_nominal_speed = MetersSeconds(0.50);
        let fight_duration = DEFAULT_FIGHT_DURATION;
        let morale_end_threshold = 20.;

        Self {
            world_width,
            world_height,
            region_width,
            region_height,
            tiles_count,
            regions_count,
            regions_width,
            regions_height,
            world_width_pixels,
            world_height_pixels,
            region_width_pixels,
            region_height_pixels,
            speed,
            individual_tick,
            visibilities_tick,
            squad_tick,
            physics_tick,
            scheduler_tick,
            interiors_tick,
            geo_pixels_per_meters,
            geo_pixels_per_tile,
            geo_meters_per_z,
            minimap_width_pixels,
            minimap_height_pixels,
            formation_tiles_between_positions,
            formation_tiles_radius_placement,
            individual_visibility_until,
            shoot_visibility_bonus,
            base_inaccuracy,
            suppress_inaccuracy,
            standup_inaccuracy,
            walking_inaccuracy,
            running_inaccuracy,
            crawling_inaccuracy,
            prone_inaccuracy,
            vehicle_moving_inaccuracy,
            opacity_inaccuracy,
            opacity_inaccuracy_reference_distance,
            opacity_inaccuracy_min_distance_factor,
            opacity_inaccuracy_max_distance_factor,
            inaccuracy_down_factor,
            ignore_firsts_lov_tiles,
            ignore_firsts_physics_pixels,
            proximity_individual_rayon,
            proximity_projectile_fly_tick_increase_value,
            proximity_projectile_impact_tick_increase_value,
            impact_sound_chance,
            individual_tick_decrease_suppress,
            individual_tick_decrease_stunned,
            explosion_stunned_value,
            vehicle_crew_stunned_value,
            individual_suppress_limit_hide,
            individual_hide_engage_distance,
            individual_grenade_enemy_distance,
            hide_opacity_bonus_ends,
            hide_opacity_bonus_meter_factor,
            tiles_radius_opacity_picking,
            walk_nominal_speed,
            run_nominal_speed,
            crawl_nominal_speed,
            fight_duration,
            morale_end_threshold,
        }
    }

    pub fn speed(&self) -> f32 {
        #[cfg(feature = "debug")]
        return (SPEED_RAW.load(Ordering::Relaxed) as f32 / 10.).max(0.1);

        #[cfg(not(feature = "debug"))]
        self.speed
    }

    pub fn with_region_width(mut self, value: u64) -> Self {
        self.region_width = value;
        self.regions_count = self.tiles_count / (self.region_width * self.region_height);
        self.regions_width = self.world_width / self.region_width();
        self.region_width_pixels = self.region_width * self.geo_pixels_per_tile;
        self
    }

    pub fn with_region_height(mut self, value: u64) -> Self {
        self.region_height = value;
        self.regions_count = self.tiles_count / (self.region_width * self.region_height);
        self.regions_height = self.world_height / self.region_height();
        self.region_height_pixels = self.region_height * self.geo_pixels_per_tile;
        self
    }

    pub fn individual_tick(&self) -> Frequency {
        self.individual_tick * self.speed()
    }

    pub fn visibilities_tick(&self) -> Frequency {
        self.visibilities_tick * self.speed()
    }

    pub fn squad_tick(&self) -> Frequency {
        self.squad_tick * self.speed()
    }

    pub fn physics_tick(&self) -> Frequency {
        self.physics_tick * self.speed()
    }

    pub fn scheduler_tick(&self) -> Frequency {
        self.scheduler_tick * self.speed()
    }

    #[cfg(feature = "debug")]
    pub fn inaccuracy_spread() -> f32 {
        INACCURACY_SPREAD_RAW.load(Ordering::Relaxed) as f32 / 10_000.
    }

    #[cfg(feature = "debug")]
    pub fn set_inaccuracy_spread(value: f32) {
        INACCURACY_SPREAD_RAW.store((value * 10_000.) as u32, Ordering::Relaxed);
    }
    #[cfg(feature = "debug")]
    pub fn inaccuracy_spread_enabled() -> bool {
        INACCURACY_SPREAD_ENABLED.load(Ordering::Relaxed)
    }

    #[cfg(feature = "debug")]
    pub fn set_inaccuracy_spread_enabled(value: bool) {
        INACCURACY_SPREAD_ENABLED.store(value, Ordering::Relaxed);
    }

    #[cfg(feature = "debug")]
    pub fn immortality() -> bool {
        IMMORTALITY.load(Ordering::Relaxed)
    }

    #[cfg(feature = "debug")]
    pub fn set_immortality(value: bool) {
        IMMORTALITY.store(value, Ordering::Relaxed);
    }

    #[cfg(feature = "debug")]
    pub fn set_speed(value: u32) {
        SPEED_RAW.store(value, Ordering::Relaxed);
    }
}

pub trait Client: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static {}
impl<T: Clone + std::hash::Hash + Eq + std::fmt::Debug + Send + Sync + 'static> Client for T {}

pub trait WcfgFrom<T>: Sized {
    fn from_(value: T, w: &WorldConfig) -> Self;
}

pub trait WcfgInto<T>: Sized {
    fn into_(self, w: &WorldConfig) -> T;
}

impl<T, U> WcfgInto<U> for T
where
    U: WcfgFrom<T>,
{
    fn into_(self, w: &WorldConfig) -> U {
        U::from_(self, w)
    }
}

#[cfg(feature = "bevy")]
#[derive(Debug, Resource, Deref, Default)]
pub struct Wcfg(pub Option<WorldConfig>);
