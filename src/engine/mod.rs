use ggez::event;
use ggez::graphics::{self};
use ggez::timer::check_update_time;
use ggez::{Context, GameResult};
use glam::*;

use crate::config::Config;
use crate::graphics::Graphics;
use crate::message::*;
use crate::network::Network;
use crate::state::State;
mod animate;
mod draw;
mod entity;
mod network;
mod react;
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
            crate::NetWorkMode::Server => self.state.init()?,
            // Client initialize its state when received from server
            crate::NetWorkMode::Client => {}
        };
        self.network.init()?;
        Ok(())
    }

    fn tick(&mut self) {
        // Will collect all tick messages
        let mut messages = vec![];

        messages.extend(self.tick_entities());
        messages.extend(self.sync());

        // FIXME HARD CODED fo test network (TODO  : in update init ?)
        match self.config.network_mode() {
            crate::NetWorkMode::Server => {
                //
            }
            crate::NetWorkMode::Client => {
                if self.frame_i == 0 {
                    self.network
                        .send(Message::Network(NetworkMessage::RequireCompleteSync));
                }
            }
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
