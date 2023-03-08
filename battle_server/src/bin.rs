use crossbeam_channel::unbounded;
use env_logger::Env;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use battle_core::config::ServerConfig;
use battle_core::network::error::NetworkError;
use battle_core::network::server::Server;
use battle_core::state::battle::builder::{BattleStateBuilder, BattleStateBuilderError};
use structopt::StructOpt;

use battle_server::runner::{Runner, RunnerError};

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(long = "rep-address")]
    rep_address: String,

    #[structopt(long = "bind-address")]
    pub_address: String,

    #[structopt(long = "profile")]
    profile: bool,

    #[structopt(long = "--profile-address", default_value = "0.0.0.0:8585")]
    profile_address: String,
}

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let stop_required = Arc::new(AtomicBool::new(false));
    let opt = Opt::from_args();
    let resources = PathBuf::from("./resources");
    let map_name = "map1";
    let situation = "hardcode";

    let _puffin_server = if opt.profile {
        let puffin_server = puffin_http::Server::new(&opt.profile_address).unwrap();
        puffin::set_scopes_on(true);
        Some(puffin_server)
    } else {
        None
    };

    let (server_input_sender, server_input_receiver) = unbounded();
    let (server_output_sender, server_output_receiver) = unbounded();
    let stop_required_ = stop_required.clone();
    let server = Server::new(
        opt.rep_address.clone(),
        opt.pub_address.clone(),
        server_output_receiver,
        server_input_sender,
        stop_required_,
    );
    server.serve()?;

    let stop_required_ = stop_required.clone();
    let config = ServerConfig::new();
    let battle_state = BattleStateBuilder::new(map_name, &resources)?
        .situation(situation)
        .build();
    let mut runner = Runner::new(
        config,
        server_input_receiver,
        server_output_sender,
        stop_required_,
        battle_state,
    );

    runner.run()?;
    Ok(())
}

#[derive(Debug)]
enum Error {
    LoadBattle(BattleStateBuilderError),
    Network(NetworkError),
    Run(RunnerError),
}

impl From<RunnerError> for Error {
    fn from(error: RunnerError) -> Self {
        Self::Run(error)
    }
}

impl From<NetworkError> for Error {
    fn from(error: NetworkError) -> Self {
        Self::Network(error)
    }
}

impl From<BattleStateBuilderError> for Error {
    fn from(error: BattleStateBuilderError) -> Self {
        Self::LoadBattle(error)
    }
}
