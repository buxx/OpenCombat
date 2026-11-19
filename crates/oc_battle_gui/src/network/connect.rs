use std::sync::{Arc, Mutex, mpsc::channel};

use bevy::prelude::*;
use oc_root::identity::Identity;

use crate::network::state::State;
use crate::network::{input::NetworkMessageReceiver, start_network};
use crate::replay::Replay;
use crate::states::AppState;
use crate::{error::OkOrSendError, network::output::ToServerSender};

#[derive(Event)]
pub struct Connect(pub crate::config::Connect, pub Identity);

#[derive(Event)]
pub struct Connected;

// TODO: manage reconnection
#[derive(Event)]
pub struct Disconnected;

// TODO: display it in gui
#[derive(Event)]
pub struct FailedToConnect;

pub fn on_connect(
    event: On<Connect>,
    mut to_server: ResMut<ToServerSender>,
    mut network_message: ResMut<NetworkMessageReceiver>,
    mut commands: Commands,
    mut network_state: ResMut<State>,
) {
    network_state.server = Some(event.0.clone());
    network_state.identity = Some(event.1.clone());

    match &event.0 {
        crate::config::Connect::Network(socket) => {
            let (input_tx, input_rx) = channel();
            let (output_tx, output_rx) = channel();

            start_network(*socket, input_tx, output_rx).ok_or_send(commands);

            to_server.0 = Some(output_tx);
            network_message.0 = Some(Arc::new(Mutex::new(input_rx)));
        }
        crate::config::Connect::Embedded(output, input) => {
            to_server.0 = Some(output.clone());
            network_message.0 = Some(input.clone());
        }
        crate::config::Connect::Replay(replay) => {
            let (input_tx, input_rx) = channel();
            let (output_tx, output_rx) = channel();

            match crate::network::replay::start(replay.file.clone(), 1.0, input_tx, output_rx) {
                Ok(handle) => commands.insert_resource(Replay(Some(handle))),
                Err(error) => Err::<(), _>(error).ok_or_send(commands),
            }

            to_server.0 = Some(output_tx);
            network_message.0 = Some(Arc::new(Mutex::new(input_rx)));
        }
    }
}

pub fn on_connected(
    _: On<Connected>,
    mut network_state: ResMut<State>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    network_state.connected = true;
    tracing::debug!("Entering 'Downloading' state");
    *app_state = NextState::Pending(AppState::Downloading);
}
