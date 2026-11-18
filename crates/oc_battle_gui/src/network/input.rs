use std::sync::{Arc, Mutex, mpsc::Receiver};

use bevy::prelude::*;
use oc_network::ToClient;
use oc_utils::let_some;

use crate::network::connect::{Connected, Disconnected, FailedToConnect};

#[derive(Debug, Event)]
pub struct NetWorkMessageEvent(pub NetworkMessage);

#[derive(Debug, Event)]
pub struct ToClientEvent(pub ToClient);

#[derive(Resource, Default)]
pub struct NetworkMessageReceiver(pub Option<Arc<Mutex<Receiver<NetworkMessage>>>>);

#[derive(Debug)]
pub enum NetworkMessage {
    Connected,
    FailToConnect,
    Disconnected,
    Message(Box<ToClient>),
}

pub fn network_message_router(mut commands: Commands, messages: Res<NetworkMessageReceiver>) {
    let_some!(messages = &messages.0, return);
    let messages = messages.lock().expect("Assume mutex");
    let messages = messages.try_iter();

    for message in messages {
        commands.trigger(NetWorkMessageEvent(message))
    }
}

pub fn on_network_message(message: On<NetWorkMessageEvent>, mut commands: Commands) {
    match &message.0 {
        NetworkMessage::FailToConnect => {
            commands.trigger(FailedToConnect);
        }
        NetworkMessage::Connected => {
            commands.trigger(Connected);
        }
        NetworkMessage::Disconnected => {
            commands.trigger(Disconnected);
        }
        NetworkMessage::Message(message) => {
            commands.trigger(ToClientEvent((**message).clone()));
        }
    }
}
