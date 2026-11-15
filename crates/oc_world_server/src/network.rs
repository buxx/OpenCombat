use oc_root::format::{decode, encode};
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender, channel};

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self};
use oc_individual::IndividualIndex;
use oc_network::{ToClient, ToServer};
use oc_projectile::ProjectileId;

use crate::bridge::Event;

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub host: SocketAddr,
    // pub static_: SocketAddr,
    // pub cache: PathBuf,
}

pub fn listen(config: NetworkConfig) -> (Receiver<Event<Endpoint>>, Sender<(Endpoint, ToClient)>) {
    let (input_tx, input_rx) = channel();
    let (output_tx, output_rx) = channel();

    std::thread::spawn(move || listen_(config.host, input_tx, output_rx));

    (input_rx, output_tx)
}

fn listen_(
    host: SocketAddr,
    input: Sender<Event<Endpoint>>,
    output: Receiver<(Endpoint, ToClient)>,
) {
    tracing::info!("Start listening on {}", host);
    let (handler, listener) = node::split::<()>();

    handler
        .network()
        .listen(Transport::FramedTcp, host)
        .unwrap(); // TODO

    std::thread::spawn(move || {
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(_, _) => unreachable!(),
            NetEvent::Accepted(endpoint, _listener) => {
                tracing::debug!("Client connected from {}", endpoint.addr());
                input.send(Event::Connected(endpoint)).unwrap(); // TODO
            }
            NetEvent::Message(endpoint, bytes_) => {
                let message = decode::<ToServer>(bytes_).unwrap(); // TODO
                tracing::trace!(name="network-message-received", endpoint = ?endpoint, message = ?message);
                input.send(Event::Message(endpoint, message)).unwrap(); // TODO
            }
            NetEvent::Disconnected(endpoint) => {
                tracing::debug!("Client {} disconnected", endpoint.addr());
                input.send(Event::Disconnected(endpoint)).unwrap(); // TODO
            }
        });
    });

    while let Ok((endpoint, message)) = output.recv() {
        tracing::trace!(name="network-message-send", endpoint = ?endpoint, message = ?message);
        let bytes = encode(&message).unwrap(); // TODO
        handler.network().send(endpoint, &bytes);
    }

    tracing::debug!("Exit from listening");
}

pub trait IntoNetworkUpdate {
    fn to_network_update(&self, update: oc_physics::update::Update) -> impl Clone + Into<ToClient>;
}

// TODO: Derive on IndividualIndex
impl IntoNetworkUpdate for IndividualIndex {
    fn to_network_update(&self, update: oc_physics::update::Update) -> impl Clone + Into<ToClient> {
        oc_individual::network::Individual::Physics(*self, update)
    }
}

// TODO: Derive on ProjectileId
impl IntoNetworkUpdate for ProjectileId {
    fn to_network_update(&self, update: oc_physics::update::Update) -> impl Clone + Into<ToClient> {
        oc_projectile::network::Projectile::Physics(*self, update)
    }
}

pub trait IntoNetworkInsert<I> {
    fn to_network_insert(&self, i: I) -> impl Clone + Into<ToClient>;
}

// TODO: Derive on oc_individual::Individual
impl IntoNetworkInsert<IndividualIndex> for oc_individual::Individual {
    fn to_network_insert(&self, i: IndividualIndex) -> impl Clone + Into<ToClient> {
        oc_individual::network::Individual::Insert(i, self.clone())
    }
}

// TODO: Derive on oc_projectile::Projectile
impl IntoNetworkInsert<ProjectileId> for oc_projectile::Projectile {
    fn to_network_insert(&self, i: ProjectileId) -> impl Clone + Into<ToClient> {
        oc_projectile::network::Projectile::Insert(i, self.clone())
    }
}

pub trait IntoNetworkForgot<I> {
    fn to_network_forgot(&self, i: I) -> impl Clone + Into<ToClient>;
}

// TODO: Derive on oc_individual::Individual
impl IntoNetworkForgot<IndividualIndex> for oc_individual::Individual {
    fn to_network_forgot(&self, i: IndividualIndex) -> impl Clone + Into<ToClient> {
        oc_individual::network::Individual::Forgot(i)
    }
}

// TODO: Derive on oc_projectile::Projectile
impl IntoNetworkForgot<ProjectileId> for oc_projectile::Projectile {
    fn to_network_forgot(&self, i: ProjectileId) -> impl Clone + Into<ToClient> {
        oc_projectile::network::Projectile::Forgot(i)
    }
}
