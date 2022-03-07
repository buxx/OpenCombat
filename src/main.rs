use std::path;

use ggez::event;
use ggez::GameResult;
use state::State;

mod behavior;
mod config;
mod engine;
mod entity;
mod graphics;
mod hardcode;
mod message;
mod network;
mod order;
mod state;
mod sync;
mod types;
mod utils;

use structopt::clap::arg_enum;
use structopt::StructOpt;

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
        .add_resource_path(path::PathBuf::from("./resources"));
    let (mut context, event_loop) = context_builder.build()?;

    let graphics = graphics::Graphics::new(&mut context)?;
    let state = match config.network_mode() {
        NetWorkMode::Server => {
            let entities = hardcode::get_entities();
            State::new(entities)
        }
        NetWorkMode::Client => State::new(vec![]),
    };
    let engine = engine::Engine::new(config, graphics, state)?;
    event::run(context, event_loop, engine)
}
