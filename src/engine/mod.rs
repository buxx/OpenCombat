use ggez::graphics::{self};
use ggez::timer::check_update_time;
use ggez::{event, GameError};
use ggez::{Context, GameResult};
use glam::*;

use crate::config::Config;
use crate::graphics::Graphics;
use crate::network::Network;
use crate::state::State;
mod animate;
mod client;
mod draw;
mod entity;
mod input;
mod network;
mod order;
mod react;
mod server;
mod update;

pub struct Engine {
    config: Config,
    network: Network,
    graphics: Graphics,
    state: State,
    frame_i: u64,
}

impl Engine {
    pub fn new(config: Config, graphics: Graphics, state: State) -> GameResult<Engine> {
        let network = Network::new(config.clone())?;
        let engine = Engine {
            config,
            network,
            graphics,
            state,
            frame_i: 0,
        };
        Ok(engine)
    }

    fn init(&mut self) -> GameResult {
        match self.config.network_mode() {
            // Server own game state, so init it
            crate::NetWorkMode::Server => self.state.init()?,
            // Client initialize its state when received from server
            crate::NetWorkMode::Client => {}
        };

        if let Err(error) = self.network.init() {
            return Err(GameError::CustomError(error.to_string()));
        }

        Ok(())
    }

    fn tick(&mut self) {
        match self.config.network_mode() {
            crate::NetWorkMode::Server => self.tick_as_server(),
            crate::NetWorkMode::Client => self.tick_as_client(),
        }
    }
}

impl event::EventHandler<ggez::GameError> for Engine {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, self.config.target_fps()) {
            // First thing to do is to initialize the state.
            if self.frame_i == 0 {
                self.init()?;
            }

            // Execute "each frame" code
            self.tick();

            // Increment the frame counter
            self.frame_i += 1;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.graphics.clear(ctx);

        for entity in self.state.entities() {
            self.graphics.extend(self.entity_sprites(entity)?);
        }

        // TODO See in OC1
        let window_draw_param = graphics::DrawParam::new()
            .dest(Vec2::new(0., 0.))
            .scale(Vec2::new(1., 1.));

        // Draw entities
        self.graphics.draw(ctx, window_draw_param)?;

        graphics::present(ctx)?;
        Ok(())
    }
}
