use entity::Entity;
use ggez::event;
use ggez::GameResult;
use state::State;
use types::*;

mod config;
mod engine;
mod entity;
mod message;
mod state;
mod types;

fn main() -> GameResult {
    let context_builder = ggez::ContextBuilder::new("OpenCombat", "Bastien Sevajol");
    let (context, event_loop) = context_builder.build()?;

    let config = config::Config::new();
    let entities: Vec<Box<dyn Entity + Send + Sync>> = vec![
        Box::new(entity::soldier::Soldier::new(WorldPosition::from((
            WorldX::from(10.),
            WorldY::from(10.),
        )))),
        Box::new(entity::soldier::Soldier::new(WorldPosition::from((
            WorldX::from(20.),
            WorldY::from(20.),
        )))),
    ];
    let state = State::new(entities);
    let engine = engine::Engine::new(config, state)?;
    event::run(context, event_loop, engine)
}
