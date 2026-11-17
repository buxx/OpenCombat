use std::{
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
};

use bevy::prelude::*;
use bon::Builder;
use oc_network::ToServer;
use oc_root::{
    files::{Connection, Roots},
    side::Side,
};

use crate::network::input::NetworkMessage;

#[derive(Debug, Clone, Default, Builder)]
pub struct Config_ {
    #[builder(default = 10)]
    pub regions_width: u64,
    #[builder(default = 10)]
    pub regions_height: u64,
    pub auto_connect: Option<(Connect, Side)>,
    /// Where files are (embedded: server's ones, network: local download directory)
    #[builder(default)]
    pub roots: Roots,
    #[builder(default)]
    pub see_all_enemies: bool,
}

#[derive(Debug, Clone)]
pub enum Connect {
    Network(SocketAddr),
    Embedded(Sender<ToServer>, Arc<Mutex<Receiver<NetworkMessage>>>),
}

impl From<Connect> for Connection {
    fn from(value: Connect) -> Self {
        match value {
            Connect::Network(addr) => Connection::Network(addr),
            Connect::Embedded(_, _) => Connection::Embedded,
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Config(pub Config_);
