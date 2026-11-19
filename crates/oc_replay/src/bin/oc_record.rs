use std::{net::SocketAddr, path::PathBuf};

use anyhow::Context;
use clap::Parser;
use message_io::{
    network::{NetEvent, Transport},
    node::{self, NodeEvent},
};
use oc_network::{ToClient, ToServer};
use oc_replay::{Recorder, Source};
use oc_root::{
    format::{decode, encode},
    side::Side,
};
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

#[derive(Parser, Debug)]
#[command(version, about = "Record a battle from a world server", long_about = None)]
struct Args {
    /// World server address (like 127.0.0.1:6589)
    address: SocketAddr,
    /// Record file path to write
    output: PathBuf,
    /// Side
    #[clap(long, value_enum, default_value = "a")]
    side: Side,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();

    let mut recorder = Recorder::new(&args.output, args.side, Source::Server(args.address));

    let (handler, listener) = node::split::<()>();
    let (server, _) = handler
        .network()
        .connect(Transport::FramedTcp, args.address)
        .with_context(|| format!("Connect to {}", args.address))?;

    {
        let handler = handler.clone();
        ctrlc::set_handler(move || {
            tracing::info!("Stop recording");
            handler.stop();
        })?;
    }

    let send = |message: ToServer| match encode(&message) {
        Ok(bytes) => {
            handler.network().send(server, &bytes);
        }
        Err(error) => tracing::error!("Encode {message:?}: {error}"),
    };

    listener.for_each(|event| {
        let NodeEvent::Network(event) = event else {
            return;
        };

        match event {
            NetEvent::Connected(_, true) => tracing::info!("Connected to {}", args.address),
            NetEvent::Connected(_, false) => {
                tracing::error!("Failed to connect {}", args.address);
                handler.stop();
            }
            NetEvent::Accepted(_, _) => {}
            NetEvent::Message(_, bytes) => {
                let message = match decode::<ToClient>(bytes) {
                    Ok(message) => message,
                    Err(error) => {
                        tracing::error!("Decode message from server: {error}");
                        return;
                    }
                };

                match recorder.record(&message) {
                    Ok(replies) => replies.into_iter().for_each(&send),
                    Err(error) => {
                        tracing::error!("Write record: {error}");
                        handler.stop();
                    }
                }
                if recorder.ended() {
                    handler.stop();
                }
            }
            NetEvent::Disconnected(_) => {
                tracing::info!("Disconnected from server");
                handler.stop();
            }
        }
    });

    recorder.flush()?;
    let count = recorder.count();
    tracing::info!("{count} messages written in {}", args.output.display());

    Ok(())
}
