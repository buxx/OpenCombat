use std::sync::mpsc::Sender;

use bevy::prelude::*;
use oc_network::ToServer;
use oc_utils::error::OkOrLogError;

#[derive(Resource, Default)]
pub struct ToServerSender(pub Option<Sender<ToServer>>);

#[derive(Debug, Event)]
pub struct ToServerEvent(pub ToServer);

impl From<ToServer> for ToServerEvent {
    fn from(value: ToServer) -> Self {
        Self(value)
    }
}

pub fn on_to_server(message: On<ToServerEvent>, sender: Res<ToServerSender>) {
    let Some(sender) = &sender.0 else { return };
    sender.send(message.0.clone()).ok_or_log();
}
