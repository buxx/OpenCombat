use std::path;

use game::Side;
use ggez::event;
use ggez::GameResult;
use state::shared::SharedState;

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

pub const RESOURCE_PATH: &'static str = "resources";

arg_enum! {
    #[derive(Debug, Clone)]
    pub enum NetWorkMode {
        Server,
        Client
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct Opt {
    #[structopt(possible_values = &NetWorkMode::variants(), case_insensitive = true)]
    network_mode: NetWorkMode,

    #[structopt(long = "--server-rep-address")]
    server_rep_address: String,

    #[structopt(long = "--server-bind-address")]
    server_pub_address: String,
}

fn main() -> GameResult {
    let opt = Opt::from_args();
    let config = config::Config::new(&opt)?;

    let context_builder = ggez::ContextBuilder::new("OpenCombat", "Bastien Sevajol")
        .add_resource_path(path::PathBuf::from(format!("./{}", RESOURCE_PATH)));
    let (mut context, event_loop) = context_builder.build()?;

    let map = map::Map::new("map1")?;
    let graphics = graphics::Graphics::new(&mut context, &map)?;
    let shared_state = match config.network_mode() {
        NetWorkMode::Server => {
            let entities = hardcode::get_entities();
            SharedState::new(entities)
        }
        NetWorkMode::Client => SharedState::new(vec![]),
    };
    let engine = engine::Engine::new(config, graphics, shared_state, Side::A, map)?;
    event::run(context, event_loop, engine)
}
