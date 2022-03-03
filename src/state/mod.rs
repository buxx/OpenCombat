use ggez::event;
use ggez::graphics::{self, Color};
use ggez::timer::check_update_time;
use ggez::{Context, GameResult};
use glam::*;

use crate::config::Config;
use crate::entity::Entity;
mod react;
mod tick;

pub struct MainState {
    config: Config,
    entities: Vec<Box<dyn Entity + Send + Sync>>,
    frame_i: u64,
}

impl MainState {
    pub fn new(
        config: Config,
        entities: Vec<Box<dyn Entity + Send + Sync>>,
    ) -> GameResult<MainState> {
        let state = MainState {
            config,
            entities,
            frame_i: 0,
        };
        Ok(state)
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, self.config.target_fps()) {
            self.frame_i += 1;
            self.tick()
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        for entity in &self.entities {
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
