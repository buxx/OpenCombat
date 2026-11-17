// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

pub mod ai;
pub mod args;
pub mod bridge;
pub mod config;
pub mod end;
pub mod generate;
pub mod index;
pub mod individual;
pub mod network;
pub mod outline;
#[cfg(feature = "perfs")]
pub mod perf;
pub mod physics;
pub mod projectile;
pub mod routing;
pub mod runner;
pub mod schedule;
pub mod squad;
pub mod state;
pub mod static_;
#[cfg(feature = "tracker")]
pub mod tracker;
pub mod utils;
pub mod visibility;
pub mod world;
