pub mod individual;
pub mod world;

pub const Z_REGION_BACKGROUND: f32 = 0.0;
pub const Z_INTERIOR: f32 = 0.05;
pub const Z_FLAG: f32 = 0.07;
#[cfg(feature = "debug")]
pub const Z_TERRAIN_TILE: f32 = 0.1;
pub const Z_CRATER: f32 = 0.2;
pub const Z_ALTERED_TILE: f32 = 0.25;
pub const Z_INDIVIDUAL: f32 = 2.0;
pub const Z_PROJECTILE: f32 = 3.0;
pub const Z_DECOR: f32 = 3.5;
pub const Z_EXPLOSION: f32 = 3.5;
pub const Z_SPAWN_ZONE: f32 = 4.0;
pub const Z_SELECT_WIRES: f32 = 5.0;
#[cfg(feature = "debug")]
pub const Z_REGION_WIREFRAME: f32 = 1.0;
pub const Z_LOV: f32 = 10.0;
pub const Z_PATH: f32 = 9.0;
pub const Z_INDIV_ORDER: f32 = 9.1;
pub const Z_SQUAD_ORDER: f32 = 9.2;
#[cfg(feature = "debug")]
pub const Z_WORLD_CURSOR: f32 = 1.0;
pub const Z_WORLD_MAP_BACKGROUND: f32 = 0.0;
pub const Z_WORLD_MAP_MINIMAP: f32 = 0.1;
pub const Z_WORLD_MAP_BATTLE_SQUARE: f32 = 2.0;

pub const UI_FILE: &str = "ui/ui.png";
