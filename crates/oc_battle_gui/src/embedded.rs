use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, RecvTimeoutError, Sender, channel},
    },
    time::Duration,
};

use anyhow::Context;
use bon::Builder;
use oc_network::{ToClient, ToServer};
use oc_replay::{Recorder, Source};
use oc_root::{files::Roots, side::Side, static_::StaticSource};
use oc_utils::error::OkOrLogError;
use oc_world_server::{args::ServerArgs, bridge::Event, runner::Runner, state};

use crate::{config::Connect, network::input::NetworkMessage};

// Timeout on server message read to be able to write battle record
const RECEIVE_TIMEOUT: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Builder)]
pub struct Embedded {
    args: ServerArgs,
    /// Side played by gui
    #[builder(default = Side::A)]
    side: Side,
    /// Record the battle in this file
    record: Option<PathBuf>,
}

/// Embedded server clients (to differenciate how to send them events)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Peer {
    Gui,
    Recorder,
}

impl Embedded {
    /// Start the server (and wait it is ready). Return files roots and the connection to give to the gui.
    pub fn start(self) -> anyhow::Result<(Roots, Connect)> {
        // Server and gui share files (see `ServerArgs::roots`)
        let (config, map) = self
            .args
            .server_config(StaticSource::Local, Some(self.side))?;
        let roots = config.roots.clone();
        let state = Arc::new(state::init::<Peer>(config.clone(), &map)?);
        let recorder = self.record.as_ref().map(|path| {
            tracing::info!("Record battle in {}", path.display());
            Recorder::new(path, self.side, Source::Local(roots.clone()))
        });

        let (ready_tx, ready_rx) = channel::<Result<(), String>>();
        // Server threads end with the process (when gui exits)
        let (_, stop_rx) = channel::<()>();
        let (to_client_tx, to_client_rx) = channel();
        let (to_server_tx, to_server_rx) = channel();

        tracing::info!("Start embedded server");
        let (side, end, ai) = (self.side, self.args.end.clone(), self.args.ai());
        std::thread::spawn(move || {
            Runner::embedded(config, state, to_client_tx, Peer::Gui, side, end)
                .with_ais(ai)
                .run(to_server_rx, ready_tx, stop_rx);
        });

        tracing::info!("Waiting embedded server ...");
        ready_rx
            .recv()
            .context("Wait server ready")?
            .map_err(|e| anyhow::anyhow!("Server failed to start: {e}"))?;

        let (to_server, from_server) = bridge(to_client_rx, to_server_tx, recorder);
        let connect = Connect::Embedded(to_server, Arc::new(Mutex::new(from_server)));

        Ok((roots, connect))
    }
}

/// Adapt server channels to gui ones (and to recorder, if any)
fn bridge(
    server_rx: Receiver<(Peer, ToClient)>,
    server_tx: Sender<Event<Peer>>,
    recorder: Option<Recorder>,
) -> (Sender<ToServer>, Receiver<NetworkMessage>) {
    let recorder = recorder.map(|recorder| RecorderPeer::connect(recorder, server_tx.clone()));
    let to_server = forward_gui_messages(server_tx);
    let from_server = dispatch_server_messages(server_rx, recorder);
    (to_server, from_server)
}

/// Forward gui messages to server
fn forward_gui_messages(server_tx: Sender<Event<Peer>>) -> Sender<ToServer> {
    let (gui_tx, gui_rx) = channel::<ToServer>();
    std::thread::spawn(move || {
        let _ = server_tx.send(Event::Connected(Peer::Gui));
        for message in gui_rx {
            let _ = server_tx.send(Event::Message(Peer::Gui, message));
        }
    });
    gui_tx
}

/// Dispatch server messages to gui or recorder
fn dispatch_server_messages(
    server_rx: Receiver<(Peer, ToClient)>,
    mut recorder: Option<RecorderPeer>,
) -> Receiver<NetworkMessage> {
    let (gui_tx, gui_rx) = channel::<NetworkMessage>();
    std::thread::spawn(move || {
        let _ = gui_tx.send(NetworkMessage::Connected);
        loop {
            match server_rx.recv_timeout(RECEIVE_TIMEOUT) {
                Ok((Peer::Gui, message)) => {
                    let _ = gui_tx.send(NetworkMessage::Message(Box::new(message)));
                }
                Ok((Peer::Recorder, message)) => {
                    if let Some(Err(error)) = recorder.as_mut().map(|r| r.receive(&message)) {
                        tracing::error!("Record stopped: {error}");
                        recorder = None;
                    }
                }
                // Write record when idle: process can exit at any moment (when gui closes)
                Err(RecvTimeoutError::Timeout) => {
                    if let Some(recorder) = &mut recorder {
                        recorder.flush();
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    gui_rx
}

/// Recorder as an embedded server client
struct RecorderPeer {
    recorder: Recorder,
    server_tx: Sender<Event<Peer>>,
}

impl RecorderPeer {
    fn connect(recorder: Recorder, server_tx: Sender<Event<Peer>>) -> Self {
        let _ = server_tx.send(Event::Connected(Peer::Recorder));
        Self {
            recorder,
            server_tx,
        }
    }

    fn receive(&mut self, message: &ToClient) -> Result<(), oc_replay::Error> {
        for reply in self.recorder.record(message)? {
            let _ = self.server_tx.send(Event::Message(Peer::Recorder, reply));
        }
        Ok(())
    }

    fn flush(&mut self) {
        self.recorder.flush().ok_or_log();
    }
}
