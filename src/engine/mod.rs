use ggez::event;
use ggez::graphics::{self, Color};
use ggez::timer::check_update_time;
use ggez::{Context, GameResult};
use glam::*;

use crate::config::Config;
use crate::state::State;
mod react;
mod tick;

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
}

impl event::EventHandler<ggez::GameError> for Engine {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, self.config.target_fps()) {
            self.frame_i += 1;
            self.tick()
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for entity in self.state.entities() {
            let circle = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Vec2::new(0.0, 0.0),
                5.0,
                2.0,
                Color::WHITE,
            )?;
            let draw_to: Vec2 = entity.world_position().into();
            graphics::draw(ctx, &circle, (draw_to,))?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
}
