use bevy::prelude::*;

pub mod minimap;
pub mod region;

#[derive(Debug, Component)]
pub struct VisibleBattleSquare;

#[derive(Debug, Component)]
pub struct WorldMapBackground;
