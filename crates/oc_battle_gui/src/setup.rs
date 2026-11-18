use bevy::prelude::*;

pub fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    spawn_camera2d(&mut commands);
}

pub fn spawn_camera2d(commands: &mut Commands) {
    commands.spawn(Camera2d);
}
