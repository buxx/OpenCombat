use bevy::prelude::*;
use oc_network::ToServer;

use crate::{
    ingame::{
        FirstIngameEnter,
        world::{SpawnMinimap, SpawnVisibleBattleSquare, SpawnWorldMapBackground},
    },
    network::output::ToServerEvent,
};

pub fn init(mut commands: Commands) {
    tracing::debug!("Init InGame");
    commands.insert_resource(super::state::State::default());
    commands.trigger(FirstIngameEnter);
}

pub fn refresh(mut commands: Commands) {
    tracing::debug!("Trigger ToServerEvent(ToServer::Refresh)");
    commands.trigger(ToServerEvent(ToServer::Refresh));
}

pub fn spawn_world_map(mut commands: Commands) {
    tracing::debug!("Trigger SpawnMinimap");
    commands.trigger(SpawnMinimap);
    tracing::debug!("Trigger SpawnWorldMapBackground");
    commands.trigger(SpawnWorldMapBackground);
    tracing::debug!("Trigger SpawnVisibleBattleSquare");
    commands.trigger(SpawnVisibleBattleSquare);
}
