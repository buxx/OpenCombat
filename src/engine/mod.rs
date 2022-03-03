use ggez::event;
use ggez::graphics::{self, Color};
use ggez::timer::check_update_time;
use ggez::{Context, GameResult};
use glam::*;
use rayon::prelude::*;

use crate::config::Config;
use crate::message::Message;
use crate::state::State;
mod animate;
mod react;
mod update;

pub struct Engine {
    config: Config,
    frame_i: u64,
    state: State,
}

impl Engine {
    pub fn new(config: Config, state: State) -> GameResult<Engine> {
        let engine = Engine {
            config,
            frame_i: 0,
            state,
        };
        Ok(engine)
    }

    fn tick(&mut self) {
        // Will collect all tick messages
        let mut messages = vec![];

        // Entities animation
        if self.frame_i % self.config.entity_animate_freq() == 0 {
            let entity_messages: Vec<Message> = (0..self.state.entities().len())
                .into_par_iter()
                .flat_map(|i| self.animate_entity(i))
                .collect();
            messages.extend(entity_messages);
        }

        // Entities updates
        if self.frame_i % self.config.entity_update_freq() == 0 {
            let entity_messages: Vec<Message> = (0..self.state.entities().len())
                .into_par_iter()
                .flat_map(|i| self.update_entity(i))
                .collect();
            messages.extend(entity_messages);
        }

        // Apply messages
        self.react(messages);
    }
}

impl event::EventHandler<ggez::GameError> for Engine {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, self.config.target_fps()) {
            // First thing to do is to initialize the state.
            if self.frame_i == 0 {
                self.state.initialize();
            }
            self.tick();

            // Increment the frame counter
            self.frame_i += 1;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        // FIXME demo code
        for entity in self.state.entities() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Vec2::new(0.0, 0.0),
                5.0,
                2.0,
                Color::WHITE,
            )?;
            let draw_to: Vec2 = entity.get_world_position().into();
            graphics::draw(ctx, &circle, (draw_to,))?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}
