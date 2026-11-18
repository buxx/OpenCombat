use bevy::prelude::*;

use crate::states::AppState;

mod init;

pub struct HomePlugin;

impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Home), (init::init,));
    }
}
