use bevy::prelude::*;
use oc_root::identity::Identity;
use uuid::Uuid;

use crate::{config::Config, network::connect::Connect, states::AppState};

pub fn init(
    mut commands: Commands,
    mut config: ResMut<Config>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Some(connect) = config.0.auto_connect.take() {
        tracing::info!("Auto connect on {connect:?}");
        let identity = Identity {
            uuid: Uuid::new_v4(),
            side: connect.1,
        };
        commands.trigger(Connect(connect.0, identity));
        tracing::debug!("Entering 'Connecting' state");
        *state = NextState::Pending(AppState::Connecting);
    }
}
