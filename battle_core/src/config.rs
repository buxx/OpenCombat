use std::collections::HashMap;

use crate::{
    behavior::{Behavior, Body},
    game::explosive::ExplosiveType,
    map::terrain::TileType,
    types::Distance,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

pub const DEFAULT_SERVER_REP_ADDRESS: &str = "tcp://0.0.0.0:4255";
pub const DEFAULT_SERVER_PUB_ADDRESS: &str = "tcp://0.0.0.0:4256";
///
pub const TARGET_FPS: u64 = 60;
pub const SOLDIER_UPDATE_FREQ: u64 = 1;
pub const FLAGS_UPDATE_FREQ: u64 = 120;
pub const SOLDIER_ANIMATE_FREQ: u64 = 20;
pub const SQUAD_LEADERS_UPDATE_FREQ: u64 = 120;
pub const INTERIORS_UPDATE_FREQ: u64 = 60;
pub const VISIBILITY_UPDATE_FREQ: u64 = 60;
pub const MORALE_UPDATE_FREQ: u64 = 300;
pub const VICTORY_UPDATE_FREQ: u64 = 300;
pub const FEELING_DECREASING_FREQ: u64 = 60;
pub const PHYSICS_UPDATE_FREQ: u64 = 1;
//
pub const END_MORALE: f32 = 0.2;
///
pub const VISIBILITY_IDLE_STANDUP_MODIFIER: f32 = 0.5;
pub const VISIBILITY_IDLE_CROUCH_MODIFIER: f32 = 0.5;
pub const VISIBILITY_IDLE_LYING_MODIFIER: f32 = -0.9;
pub const VISIBILITY_MOVE_TO_MODIFIER: f32 = 1.0;
pub const VISIBILITY_MOVE_FAST_TO_MODIFIER: f32 = 2.0;
pub const VISIBILITY_SNEAK_TO_MODIFIER: f32 = VISIBILITY_IDLE_LYING_MODIFIER;
pub const VISIBILITY_DEFEND_MODIFIER: f32 = VISIBILITY_IDLE_LYING_MODIFIER;
pub const VISIBILITY_HIDE_MODIFIER: f32 = VISIBILITY_IDLE_LYING_MODIFIER;
pub const VISIBILITY_IN_VEHICLE_MODIFIER: f32 = 0.;
pub const VISIBILITY_SUPPRESS_FIRE_MODIFIER: f32 = 0.5;
pub const VISIBILITY_ENGAGE_MODIFIER: f32 = 0.5;
pub const VISIBILITY_DEAD_MODIFIER: f32 = 0.0;
pub const VISIBILITY_UNCONSCIOUS_MODIFIER: f32 = 0.0;
///
pub const TILE_TYPE_OPACITY_SHORT_GRASS: f32 = 0.0;
pub const TILE_TYPE_OPACITY_MIDDLE_GRASS: f32 = 0.008;
pub const TILE_TYPE_OPACITY_HIGH_GRASS: f32 = 0.1;
pub const TILE_TYPE_OPACITY_DIRT: f32 = 0.0;
pub const TILE_TYPE_OPACITY_CONCRETE: f32 = 0.0;
pub const TILE_TYPE_OPACITY_MUD: f32 = 0.0;
pub const TILE_TYPE_OPACITY_BRICK_WALL: f32 = 1.0;
pub const TILE_TYPE_OPACITY_TRUNK: f32 = 0.2;
pub const TILE_TYPE_OPACITY_WATER: f32 = 0.0;
pub const TILE_TYPE_OPACITY_DEEP_WATER: f32 = 0.0;
pub const TILE_TYPE_OPACITY_UNDERBRUSH: f32 = 0.015;
pub const TILE_TYPE_OPACITY_LIGHT_UNDERBRUSH: f32 = 0.010;
pub const TILE_TYPE_OPACITY_MIDDLE_WOOD_LOGS: f32 = 0.15;
pub const TILE_TYPE_OPACITY_HEDGE: f32 = 0.25;
pub const TILE_TYPE_OPACITY_MIDDLE_ROCK: f32 = 0.15;
//
pub const VISIBILITY_BY_LAST_FRAME_SHOOT: u64 = TARGET_FPS * 15;
pub const VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE: usize = 4;

// Target FPS
// Velocity of move vector
pub const MOVE_VELOCITY: f32 = 5.0 / TARGET_FPS as f32;
// Velocity of move fast vector
pub const MOVE_FAST_VELOCITY: f32 = 10.0 / TARGET_FPS as f32;
// Velocity of move hide vector
pub const MOVE_HIDE_VELOCITY: f32 = 1.5 / TARGET_FPS as f32;
// Selection square size of selected soldier
pub const DEFAULT_SELECTED_SQUARE_SIDE: f32 = 14.0;
// Half selection square size of selected soldier
pub const DEFAULT_SELECTED_SQUARE_SIDE_HALF: f32 = DEFAULT_SELECTED_SQUARE_SIDE / 2.0;
// Selection square size of selectable zone (click)
pub const SOLDIER_SELECTABLE_SQUARE_SIDE: f32 = 14.0;
// Half selection square size of selectable zone (click)
pub const SOLDIER_SELECTABLE_SQUARE_SIDE_HALF: f32 = SOLDIER_SELECTABLE_SQUARE_SIDE / 2.0;
// Frames to wait to draw the pending order path finding
pub const PENDING_ORDER_PATH_FINDING_DRAW_FRAMES: u64 = TARGET_FPS / 3;

pub const VEHICLE_DRIVE_ORIENTATION_TARGET_TOLERANCE_COEFFICIENT: f32 = 100.;
pub const VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_COEFFICIENT: f32 = 100.;
pub const VEHICLE_DRIVE_ORIENTATION_ADVANCE_TOLERANCE_DIFF: f32 = 50.;

// Grid distance to search cover point
pub const COVER_DISTANCE: i32 = 6;

// Visibility computing must consider firsts tiles differently
pub const VISIBILITY_FIRSTS: usize = 6;
pub const VISIBLE_STARTS_AT: f32 = 0.5;
// When compute visibility, configure here each pixels step of line which me considered
pub const VISIBILITY_PIXEL_STEPS: usize = 5;
// When compute coverage, configure here each pixels step of line which me considered
pub const COVERAGE_PIXEL_STEPS: usize = 5;
// When compute coverage, configure here how many tile to consider starting from soldier
pub const COVERAGE_TILE_STEPS: usize = 3;
// How many meters maximum soldier hide before shoot
pub const HIDE_MAXIMUM_RAYON: i64 = 50;
// How many frames after last proximity shoot needed before soldier go from lying to crouch when idle
pub const CAN_CROUCH_AFTER: u64 = TARGET_FPS * 60 * 5;
// How many frames after last proximity shoot needed before soldier go from crouch to standup when idle
pub const CAN_STANDUP_AFTER: u64 = TARGET_FPS * 60 * 10;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub send_debug_points: bool,
    pub flags_update_freq: u64,
    pub soldier_update_freq: u64,
    pub soldier_animate_freq: u64,
    pub squad_leaders_update_freq: u64,
    pub interiors_update_freq: u64,
    pub visibility_update_freq: u64,
    pub morale_update_freq: u64,
    pub victory_update_freq: u64,
    pub physics_update_freq: u64,
    pub feeling_decreasing_freq: u64,
    pub visibility_firsts: usize,
    pub visible_starts_at: f32,
    pub visibility_idle_standup_modifier: f32,
    pub visibility_idle_crouch_modifier: f32,
    pub visibility_idle_lying_modifier: f32,
    pub visibility_move_to_modifier: f32,
    pub visibility_move_fast_to_modifier: f32,
    pub visibility_sneak_to_modifier: f32,
    pub visibility_defend_modifier: f32,
    pub visibility_hide_modifier: f32,
    pub visibility_in_vehicle_modifier: f32,
    pub visibility_suppress_fire_modifier: f32,
    pub visibility_engage_modifier: f32,
    pub visibility_dead_modifier: f32,
    pub visibility_unconscious_modifier: f32,
    pub tile_type_opacity_short_grass: f32,
    pub tile_type_opacity_middle_grass: f32,
    pub tile_type_opacity_high_grass: f32,
    pub tile_type_opacity_dirt: f32,
    pub tile_type_opacity_concrete: f32,
    pub tile_type_opacity_mud: f32,
    pub tile_type_opacity_brick_wall: f32,
    pub tile_type_opacity_trunk: f32,
    pub tile_type_opacity_water: f32,
    pub tile_type_opacity_deep_water: f32,
    pub tile_type_opacity_underbrush: f32,
    pub tile_type_opacity_light_underbrush: f32,
    pub tile_type_opacity_middle_wood_logs: f32,
    pub tile_type_opacity_hedge: f32,
    pub tile_type_opacity_middle_rock: f32,
    pub visibility_by_last_frame_shoot: u64,
    pub visibility_by_last_frame_shoot_distance: usize,
    pub explosive_direct_death_rayon: HashMap<ExplosiveType, Distance>,
    pub explosive_regressive_death_rayon: HashMap<ExplosiveType, Distance>,
    pub explosive_regressive_injured_rayon: HashMap<ExplosiveType, Distance>,
    pub hide_maximum_rayon: Distance,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let mut explosive_direct_death_rayon = HashMap::new();
        let mut explosive_regressive_death_rayon = HashMap::new();
        let mut explosive_regressive_injured_rayon = HashMap::new();

        for explosive in ExplosiveType::iter() {
            explosive_direct_death_rayon.insert(explosive.clone(), explosive.direct_death_rayon());
            explosive_regressive_death_rayon
                .insert(explosive.clone(), explosive.regressive_death_rayon());
            explosive_regressive_injured_rayon
                .insert(explosive.clone(), explosive.regressive_injured_rayon());
        }

        Self {
            send_debug_points: false,
            /// Frequency of flags update
            flags_update_freq: FLAGS_UPDATE_FREQ,
            /// Frequency of soldier update :
            ///  - World pixel point according to movement
            ///  - ...
            soldier_update_freq: SOLDIER_UPDATE_FREQ,
            /// Frequency of soldier animation :
            ///  - Compute visibility with other soldiers
            ///  - Compute behavior against physics (explosions, gunfires, ...)
            soldier_animate_freq: SOLDIER_ANIMATE_FREQ,
            ///
            squad_leaders_update_freq: SQUAD_LEADERS_UPDATE_FREQ,
            /// Frequency of update buildings interiors visibility
            interiors_update_freq: INTERIORS_UPDATE_FREQ,
            /// Frequency of update visibility between soldiers
            visibility_update_freq: VISIBILITY_UPDATE_FREQ,
            ///
            morale_update_freq: MORALE_UPDATE_FREQ,
            ///
            victory_update_freq: VICTORY_UPDATE_FREQ,
            ///
            physics_update_freq: PHYSICS_UPDATE_FREQ,
            /// Frequency of decreasing feelings
            feeling_decreasing_freq: FEELING_DECREASING_FREQ,
            ///
            visibility_firsts: VISIBILITY_FIRSTS,
            visible_starts_at: VISIBLE_STARTS_AT,

            visibility_idle_standup_modifier: VISIBILITY_IDLE_STANDUP_MODIFIER,
            visibility_idle_crouch_modifier: VISIBILITY_IDLE_CROUCH_MODIFIER,
            visibility_idle_lying_modifier: VISIBILITY_IDLE_LYING_MODIFIER,
            visibility_move_to_modifier: VISIBILITY_MOVE_TO_MODIFIER,
            visibility_move_fast_to_modifier: VISIBILITY_MOVE_FAST_TO_MODIFIER,
            visibility_sneak_to_modifier: VISIBILITY_SNEAK_TO_MODIFIER,
            visibility_defend_modifier: VISIBILITY_DEFEND_MODIFIER,
            visibility_hide_modifier: VISIBILITY_HIDE_MODIFIER,
            visibility_in_vehicle_modifier: VISIBILITY_IN_VEHICLE_MODIFIER,
            visibility_suppress_fire_modifier: VISIBILITY_SUPPRESS_FIRE_MODIFIER,
            visibility_engage_modifier: VISIBILITY_ENGAGE_MODIFIER,
            visibility_dead_modifier: VISIBILITY_DEAD_MODIFIER,
            visibility_unconscious_modifier: VISIBILITY_UNCONSCIOUS_MODIFIER,
            visibility_by_last_frame_shoot: VISIBILITY_BY_LAST_FRAME_SHOOT,
            visibility_by_last_frame_shoot_distance: VISIBILITY_BY_LAST_FRAME_SHOOT_DISTANCE,

            tile_type_opacity_short_grass: TILE_TYPE_OPACITY_SHORT_GRASS,
            tile_type_opacity_middle_grass: TILE_TYPE_OPACITY_MIDDLE_GRASS,
            tile_type_opacity_high_grass: TILE_TYPE_OPACITY_HIGH_GRASS,
            tile_type_opacity_dirt: TILE_TYPE_OPACITY_DIRT,
            tile_type_opacity_concrete: TILE_TYPE_OPACITY_CONCRETE,
            tile_type_opacity_mud: TILE_TYPE_OPACITY_MUD,
            tile_type_opacity_brick_wall: TILE_TYPE_OPACITY_BRICK_WALL,
            tile_type_opacity_trunk: TILE_TYPE_OPACITY_TRUNK,
            tile_type_opacity_water: TILE_TYPE_OPACITY_WATER,
            tile_type_opacity_deep_water: TILE_TYPE_OPACITY_DEEP_WATER,
            tile_type_opacity_underbrush: TILE_TYPE_OPACITY_UNDERBRUSH,
            tile_type_opacity_light_underbrush: TILE_TYPE_OPACITY_LIGHT_UNDERBRUSH,
            tile_type_opacity_middle_wood_logs: TILE_TYPE_OPACITY_MIDDLE_WOOD_LOGS,
            tile_type_opacity_hedge: TILE_TYPE_OPACITY_HEDGE,
            tile_type_opacity_middle_rock: TILE_TYPE_OPACITY_MIDDLE_ROCK,

            explosive_direct_death_rayon,
            explosive_regressive_death_rayon,
            explosive_regressive_injured_rayon,

            hide_maximum_rayon: Distance::from_meters(HIDE_MAXIMUM_RAYON),
        }
    }
}

impl ServerConfig {
    pub fn soldier_update_freq(&self) -> u64 {
        self.soldier_update_freq
    }

    pub fn flags_update_freq(&self) -> u64 {
        self.flags_update_freq
    }

    pub fn soldier_animate_freq(&self) -> u64 {
        self.soldier_animate_freq
    }

    pub fn squad_leaders_update_freq(&self) -> u64 {
        self.squad_leaders_update_freq
    }

    pub fn interiors_update_freq(&self) -> u64 {
        self.interiors_update_freq
    }

    pub fn visibility_update_freq(&self) -> u64 {
        self.visibility_update_freq
    }

    pub fn morale_update_freq(&self) -> u64 {
        self.morale_update_freq
    }

    pub fn victory_update_freq(&self) -> u64 {
        self.victory_update_freq
    }

    pub fn physics_update_freq(&self) -> u64 {
        self.physics_update_freq
    }

    pub fn feeling_decreasing_freq(&self) -> u64 {
        self.feeling_decreasing_freq
    }

    pub fn visibility_behavior_modifier(&self, behavior: &Behavior) -> f32 {
        match behavior {
            Behavior::Idle(Body::StandUp) => self.visibility_idle_standup_modifier,
            Behavior::Idle(Body::Crouched) => self.visibility_idle_standup_modifier,
            Behavior::Idle(Body::Lying) => self.visibility_idle_standup_modifier,
            Behavior::Hide(_) => self.visibility_hide_modifier,
            Behavior::Defend(_) => self.visibility_defend_modifier,
            Behavior::MoveTo(_) => self.visibility_move_to_modifier,
            Behavior::MoveFastTo(_) => self.visibility_move_fast_to_modifier,
            Behavior::SneakTo(_) => self.visibility_sneak_to_modifier,
            Behavior::DriveTo(_) => self.visibility_in_vehicle_modifier,
            Behavior::RotateTo(_) => self.visibility_in_vehicle_modifier,
            Behavior::SuppressFire(_) => self.visibility_suppress_fire_modifier,
            Behavior::EngageSoldier(_) => self.visibility_engage_modifier,
            Behavior::Dead => self.visibility_dead_modifier,
            Behavior::Unconscious => self.visibility_unconscious_modifier,
        }
    }

    pub fn behavior_velocity(&self, behavior: &Behavior) -> Option<f32> {
        match behavior {
            Behavior::Idle(_) => None,
            Behavior::MoveTo(_) => Some(MOVE_VELOCITY),
            Behavior::MoveFastTo(_) => Some(MOVE_FAST_VELOCITY),
            Behavior::SneakTo(_) => Some(MOVE_HIDE_VELOCITY),
            Behavior::Defend(_) => None,
            Behavior::Hide(_) => None,
            Behavior::DriveTo(_) => None,
            Behavior::RotateTo(_) => None,
            Behavior::Dead => None,
            Behavior::Unconscious => None,
            Behavior::SuppressFire(_) => None,
            Behavior::EngageSoldier(_) => None,
        }
    }

    pub fn terrain_tile_opacity(&self, tile_type: &TileType) -> f32 {
        match tile_type {
            TileType::ShortGrass => self.tile_type_opacity_short_grass,
            TileType::MiddleGrass => self.tile_type_opacity_middle_grass,
            TileType::HighGrass => self.tile_type_opacity_middle_grass,
            TileType::Dirt => self.tile_type_opacity_dirt,
            TileType::Concrete => self.tile_type_opacity_concrete,
            TileType::Mud => self.tile_type_opacity_mud,
            TileType::BrickWall => self.tile_type_opacity_brick_wall,
            TileType::Trunk => self.tile_type_opacity_trunk,
            TileType::Water => self.tile_type_opacity_water,
            TileType::DeepWater => self.tile_type_opacity_deep_water,
            TileType::Underbrush => self.tile_type_opacity_underbrush,
            TileType::LightUnderbrush => self.tile_type_opacity_light_underbrush,
            TileType::MiddleWoodLogs => self.tile_type_opacity_middle_wood_logs,
            TileType::Hedge => self.tile_type_opacity_hedge,
            TileType::MiddleRock => self.tile_type_opacity_middle_rock,
        }
    }

    #[rustfmt::skip]
    pub fn react(&mut self, message: &ChangeConfigMessage) {
        match message {
            ChangeConfigMessage::SendDebugPoints(v) => self.send_debug_points = *v,
            ChangeConfigMessage::SoldierUpdateFreq(v) => self.soldier_update_freq = *v,
            ChangeConfigMessage::SoldierAnimateFreq(v) => self.soldier_animate_freq = *v,
            ChangeConfigMessage::InteriorsUpdateFreq(v) => self.interiors_update_freq = *v,
            ChangeConfigMessage::VisibilityUpdateFreq(v) => self.visibility_update_freq = *v,
            ChangeConfigMessage::FeelingDecreasingFreq(v) => self.feeling_decreasing_freq = *v,
            ChangeConfigMessage::VisibilityFirsts(v) => self.visibility_firsts = *v,
            ChangeConfigMessage::VisibleStartsAt(v) => self.visible_starts_at = *v,
            ChangeConfigMessage::VisibilityIdleStandupModifier(v) => self.visibility_idle_standup_modifier = *v,
            ChangeConfigMessage::VisibilityIdleCrouchModifier(v) => self.visibility_idle_crouch_modifier = *v,
            ChangeConfigMessage::VisibilityIdleLyingModifier(v) => self.visibility_idle_lying_modifier = *v,
            ChangeConfigMessage::VisibilityMoveModifier(v) => self.visibility_move_to_modifier = *v,
            ChangeConfigMessage::VisibilityMoveFastModifier(v) => self.visibility_move_fast_to_modifier = *v,
            ChangeConfigMessage::VisibilitySneakToModifier(v) => self.visibility_sneak_to_modifier = *v,
            ChangeConfigMessage::VisibilityDefendModifier(v) => self.visibility_defend_modifier = *v,
            ChangeConfigMessage::VisibilityHideModifier(v) => self.visibility_hide_modifier = *v,
            ChangeConfigMessage::VisibilityInVehicleModifier(v) => self.visibility_in_vehicle_modifier = *v,
            ChangeConfigMessage::VisibilitySuppressFireModifier(v) => self.visibility_suppress_fire_modifier = *v,
            ChangeConfigMessage::VisibilityEngageModifier(v) => self.visibility_engage_modifier = *v,
            ChangeConfigMessage::VisibilityDeadModifier(v) => self.visibility_dead_modifier = *v,
            ChangeConfigMessage::VisibilityUnconsciousModifier(v) => self.visibility_unconscious_modifier = *v,
            ChangeConfigMessage::TileTypeOpacityShortGrass(v) => self.tile_type_opacity_short_grass = *v,
            ChangeConfigMessage::TileTypeOpacityMiddleGrass(v) => self.tile_type_opacity_middle_grass = *v,
            ChangeConfigMessage::TileTypeOpacityHighGrass(v) => self.tile_type_opacity_high_grass = *v,
            ChangeConfigMessage::TileTypeOpacityDirt(v) => self.tile_type_opacity_dirt = *v,
            ChangeConfigMessage::TileTypeOpacityConcrete(v) => self.tile_type_opacity_concrete = *v,
            ChangeConfigMessage::TileTypeOpacityMud(v) => self.tile_type_opacity_mud = *v,
            ChangeConfigMessage::TileTypeOpacityBrickWall(v) => self.tile_type_opacity_brick_wall = *v,
            ChangeConfigMessage::TileTypeOpacityTrunk(v) => self.tile_type_opacity_trunk = *v,
            ChangeConfigMessage::TileTypeOpacityWater(v) => self.tile_type_opacity_water = *v,
            ChangeConfigMessage::TileTypeOpacityDeepWater(v) => self.tile_type_opacity_deep_water = *v,
            ChangeConfigMessage::TileTypeOpacityUnderbrush(v) => self.tile_type_opacity_underbrush = *v,
            ChangeConfigMessage::TileTypeOpacityLightUnderbrush(v) => self.tile_type_opacity_light_underbrush = *v,
            ChangeConfigMessage::TileTypeOpacityMiddleWoodLogs(v) => self.tile_type_opacity_middle_wood_logs = *v,
            ChangeConfigMessage::TileTypeOpacityHedge(v) => self.tile_type_opacity_hedge = *v,
            ChangeConfigMessage::TileTypeOpacityMiddleRock(v) => self.tile_type_opacity_middle_rock = *v,
            ChangeConfigMessage::VisibilityByLastFrameShot(v) => self.visibility_by_last_frame_shoot = *v,
            ChangeConfigMessage::VisibilityByLastFrameShotDistance(v) => self.visibility_by_last_frame_shoot_distance = *v,
            ChangeConfigMessage::ExplosiveDirectDeathRayon(explosive, new_distance) => {
                if let Some(distance) = self.explosive_direct_death_rayon.get_mut(explosive) {
                    distance.millimeters = new_distance.millimeters()
                }
            },
            ChangeConfigMessage::ExplosiveRegressiveDeathRayon(explosive, new_distance) => {
                if let Some(distance) = self.explosive_regressive_death_rayon.get_mut(explosive) {
                    distance.millimeters = new_distance.millimeters()
                }
            },
            ChangeConfigMessage::ExplosiveRegressiveInjuredRayon(explosive, new_distance) => {
                if let Some(distance) = self.explosive_regressive_injured_rayon.get_mut(explosive) {
                    distance.millimeters = new_distance.millimeters()
                }
            },
        }
    }
}

pub struct GuiConfig {
    pub target_fps: u32,
    pub interiors_update_freq: u64,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            target_fps: TARGET_FPS as u32,
            interiors_update_freq: INTERIORS_UPDATE_FREQ,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChangeConfigMessage {
    SendDebugPoints(bool),
    SoldierUpdateFreq(u64),
    SoldierAnimateFreq(u64),
    InteriorsUpdateFreq(u64),
    VisibilityUpdateFreq(u64),
    FeelingDecreasingFreq(u64),
    VisibilityFirsts(usize),
    VisibleStartsAt(f32),
    VisibilityIdleStandupModifier(f32),
    VisibilityIdleCrouchModifier(f32),
    VisibilityIdleLyingModifier(f32),
    VisibilityMoveModifier(f32),
    VisibilityMoveFastModifier(f32),
    VisibilitySneakToModifier(f32),
    VisibilityDefendModifier(f32),
    VisibilityHideModifier(f32),
    VisibilityInVehicleModifier(f32),
    VisibilitySuppressFireModifier(f32),
    VisibilityEngageModifier(f32),
    VisibilityDeadModifier(f32),
    VisibilityUnconsciousModifier(f32),
    TileTypeOpacityShortGrass(f32),
    TileTypeOpacityMiddleGrass(f32),
    TileTypeOpacityHighGrass(f32),
    TileTypeOpacityDirt(f32),
    TileTypeOpacityConcrete(f32),
    TileTypeOpacityMud(f32),
    TileTypeOpacityBrickWall(f32),
    TileTypeOpacityTrunk(f32),
    TileTypeOpacityWater(f32),
    TileTypeOpacityDeepWater(f32),
    TileTypeOpacityUnderbrush(f32),
    TileTypeOpacityLightUnderbrush(f32),
    TileTypeOpacityMiddleWoodLogs(f32),
    TileTypeOpacityHedge(f32),
    TileTypeOpacityMiddleRock(f32),
    VisibilityByLastFrameShot(u64),
    VisibilityByLastFrameShotDistance(usize),
    ExplosiveDirectDeathRayon(ExplosiveType, Distance),
    ExplosiveRegressiveDeathRayon(ExplosiveType, Distance),
    ExplosiveRegressiveInjuredRayon(ExplosiveType, Distance),
}
