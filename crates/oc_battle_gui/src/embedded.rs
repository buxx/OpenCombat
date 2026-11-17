use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender, channel},
};

use anyhow::Context;
use bon::Builder;
use oc_network::{ToClient, ToServer};
use oc_root::{files::Roots, side::Side, static_::StaticSource};
use oc_world_server::{args::ServerArgs, bridge::Event, runner::Runner, state};

use crate::{config::Connect, network::input::NetworkMessage};

#[derive(Debug, Clone, Builder)]
pub struct Embedded {
    args: ServerArgs,
    /// Side played by gui
    #[builder(default = Side::A)]
    side: Side,
}

impl Embedded {
    /// Start the server (and wait it is ready). Return files roots and the connection to give to the gui.
    pub fn start(self) -> anyhow::Result<(Roots, Connect)> {
        // Server and gui share files (see `ServerArgs::roots`)
        let (config, map) = self
            .args
            .server_config(StaticSource::Local, Some(self.side))?;
        let roots = config.roots.clone();
        let state = Arc::new(state::init::<()>(config.clone(), &map)?);

        let (ready_tx, ready_rx) = channel::<Result<(), String>>();
        // Server threads end with the process (when gui exits)
        let (_, stop_rx) = channel::<()>();
        let (to_client_tx, to_client_rx) = channel();
        let (to_server_tx, to_server_rx) = channel();

        tracing::info!("Start embedded server");
        let (side, end, ai) = (self.side, self.args.end.clone(), self.args.ai());
        std::thread::spawn(move || {
            Runner::embedded(config, state, to_client_tx, side, end)
                .with_ais(ai)
                .run(to_server_rx, ready_tx, stop_rx);
        });

        tracing::info!("Waiting embedded server ...");
        ready_rx
            .recv()
            .context("Wait server ready")?
            .map_err(|e| anyhow::anyhow!("Server failed to start: {e}"))?;

        let (to_server, from_server) = bridge(to_client_rx, to_server_tx);
        let connect = Connect::Embedded(to_server, Arc::new(Mutex::new(from_server)));

        Ok((roots, connect))
    }
}

/// Adapt server channels to gui ones
fn bridge(
    server_rx: Receiver<((), ToClient)>,
    client_tx: Sender<Event<()>>,
) -> (Sender<ToServer>, Receiver<NetworkMessage>) {
    let (to_server_tx, to_server_rx) = channel::<ToServer>();
    std::thread::spawn(move || {
        let _ = client_tx.send(Event::Connected(()));
        while let Ok(message) = to_server_rx.recv() {
            let _ = client_tx.send(Event::Message((), message));
        }
    });

    let (from_server_tx, from_server_rx) = channel::<NetworkMessage>();
    std::thread::spawn(move || {
        let _ = from_server_tx.send(NetworkMessage::Connected);
        while let Ok((_, message)) = server_rx.recv() {
            let _ = from_server_tx.send(NetworkMessage::Message(Box::new(message)));
        }
    });

    (to_server_tx, from_server_rx)
}
