// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use std::{net::SocketAddr, sync::Arc, sync::mpsc::channel};

use clap::Parser;
use message_io::network::Endpoint;
use oc_root::static_::StaticSource;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "tracker")]
use oc_world_server::tracker::Tracker;
use oc_world_server::{
    args::ServerArgs,
    network::{self, NetworkConfig},
    runner::Runner,
    state,
    static_::Static,
};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(long, default_value = "0.0.0.0:6589")]
    pub host: SocketAddr,

    #[clap(long("static"), default_value = "0.0.0.0:6590")]
    pub static_: SocketAddr,

    #[command(flatten)]
    pub server: ServerArgs,

    #[clap(long, action)]
    pub print_ticks: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    setup_logging()?;

    #[cfg(feature = "tracker")]
    let tracker = Tracker::default();

    let network = NetworkConfig { host: args.host };
    let (config, map) = args
        .server
        .server_config(StaticSource::Remote(args.static_.port()), None)?;
    let state = state::init::<Endpoint>(config.clone(), &map)?;
    let state = Arc::new(state);

    let (input, output) = network::listen(network.clone());
    {
        let state = state.clone();
        let config = config.clone();
        std::thread::spawn(move || Static::new(state, network.clone(), config).serve(args.static_));
    }

    let (ready, _) = channel();
    let (stop_tx, stop_rx) = channel();
    let end = args.server.end.clone();
    let ai = args.server.ai();
    std::thread::spawn(move || {
        Runner::new(
            config,
            state,
            output,
            #[cfg(feature = "tracker")]
            tracker,
            #[cfg(feature = "debug")]
            None,
            end,
        )
        .with_ais(ai)
        .run(input, ready, stop_rx);
    });

    let (tx, rx) = channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");
    tracing::info!("Waiting for SIGINT (Ctrl+C)...");
    let _ = rx.recv();

    tracing::info!("SIGINT (Ctrl+C) received, stop runner");
    let _ = stop_tx.send(());

    Ok(())
}

fn setup_logging() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();
    Ok(())
}
