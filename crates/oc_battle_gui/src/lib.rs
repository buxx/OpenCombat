// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

#[cfg(feature = "debug")]
pub mod auto_end;
pub mod config;
#[cfg(feature = "debug")]
pub mod debug;
pub mod downloading;
pub mod embedded;
pub mod entity;
pub mod error;
pub mod fx;
pub mod home;
pub mod identity;
pub mod individual;
pub mod ingame;
pub mod menu;
pub mod network;
#[cfg(feature = "debug")]
pub mod projectile;
pub mod run;
pub mod setup;
pub mod sprites;
pub mod states;
#[cfg(feature = "debug")]
pub mod tileset;
pub mod utils;
pub mod window;
pub mod world;

#[cfg(test)]
pub mod tests;
