use ggez::event;
use ggez::GameResult;
use state::State;
use types::*;

mod config;
mod engine;
mod entity;
mod message;
mod order;
mod state;
mod types;
mod utils;

fn main() -> GameResult {
    let context_builder = ggez::ContextBuilder::new("OpenCombat", "Bastien Sevajol");
    let (context, event_loop) = context_builder.build()?;

    let config = config::Config::new();

    let squad_1 = utils::squad_uuid();
    let squad_2 = utils::squad_uuid();

    let entities: Vec<ThreadSafeEntity> = vec![
        Box::new(entity::soldier::Soldier::new(
            WorldPosition::from((WorldX::from(10.), WorldY::from(10.))),
            squad_1,
        )),
        Box::new(entity::soldier::Soldier::new(
            WorldPosition::from((WorldX::from(20.), WorldY::from(20.))),
            squad_1,
        )),
        Box::new(entity::soldier::Soldier::new(
            WorldPosition::from((WorldX::from(50.), WorldY::from(20.))),
            squad_2,
        )),
    ];
    let state = State::new(entities);
    let engine = engine::Engine::new(config, state)?;
    event::run(context, event_loop, engine)
}
