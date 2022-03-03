use ggez::event;
use ggez::GameResult;
use state::State;

mod config;
mod engine;
mod entity;
mod hardcode;
mod message;
mod order;
mod state;
mod types;
mod utils;

fn main() -> GameResult {
    let context_builder = ggez::ContextBuilder::new("OpenCombat", "Bastien Sevajol");
    let (context, event_loop) = context_builder.build()?;

    let config = config::Config::new();
    let entities = hardcode::get_entities();
    let state = State::new(entities);
    let engine = engine::Engine::new(config, state)?;
    event::run(context, event_loop, engine)
}
