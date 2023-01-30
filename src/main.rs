use std::path;

use game::Side;
use ggez::conf::WindowMode;
use ggez::event;
use ggez::GameResult;
use state::local::LocalState;
use state::shared::SharedState;

mod audio;
mod behavior;
mod config;
mod debug;
mod engine;
mod entity;
mod game;
mod graphics;
mod hardcode;
mod map;
mod message;
mod network;
mod order;
mod physics;
mod state;
mod sync;
mod types;
mod ui;
mod utils;

use structopt::clap::arg_enum;
use structopt::StructOpt;
use types::*;

pub const RESOURCE_PATH: &'static str = "resources";

arg_enum! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum NetworkMode {
        Server,
        Client
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(possible_values = &NetworkMode::variants(), case_insensitive = true)]
    network_mode: NetworkMode,

    #[structopt(long = "--server-rep-address")]
    server_rep_address: String,

    #[structopt(long = "--server-bind-address")]
    server_pub_address: String,

    #[structopt(short = "s", long = "side", default_value = "a")]
    side: Side,
}

fn main() -> GameResult {
    let opt = Opt::from_args();
    let config = config::Config::new(&opt)?;

    let context_builder = ggez::ContextBuilder::new("OpenCombat", "Bastien Sevajol")
        .add_resource_path(path::PathBuf::from(format!("./{}", RESOURCE_PATH)))
        .window_mode(WindowMode::default().dimensions(1024., 768.));
    let (mut context, event_loop) = context_builder.build()?;

    let map = map::reader::Reader::new("map1")?.build()?;
    let graphics = graphics::Graphics::new(&mut context, &map)?;
    let shared_state = match config.network_mode() {
        NetworkMode::Server => {
            let (soldiers, vehicles, boards) = hardcode::shared_state_fixtures();
            SharedState::new(soldiers, vehicles, boards)
        }
        NetworkMode::Client => SharedState::new(vec![], vec![], SoldiersOnBoard::new()),
    };
    let local_state = LocalState::new(opt.side);
    let engine = engine::Engine::new(config, graphics, shared_state, local_state, map)?;
    event::run(context, event_loop, engine)
}
