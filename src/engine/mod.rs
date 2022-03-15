use ggez::event::MouseButton;
use ggez::graphics::{self, MeshBuilder};
use ggez::timer::check_update_time;
use ggez::{event, GameError};
use ggez::{Context, GameResult};

use crate::config::Config;
use crate::game::Side;
use crate::graphics::Graphics;
use crate::network::Network;
use crate::state::local::LocalState;
use crate::state::shared::SharedState;
mod animate;
mod client;
mod debug;
mod draw;
mod entity;
mod input;
mod network;
mod order;
mod react;
mod server;
mod ui;
mod update;
mod utils;

pub struct Engine {
    config: Config,
    network: Network,
    graphics: Graphics,
    /// The current shared state of the game. This struct is own by server and replicated on clients
    shared_state: SharedState,
    /// The current local state of the game. This struct is own by client and server and are not related
    local_state: LocalState,
}

impl Engine {
    pub fn new(
        config: Config,
        graphics: Graphics,
        shared_state: SharedState,
        side: Side,
    ) -> GameResult<Engine> {
        let network = Network::new(config.clone())?;
        let local_state = LocalState::new(side);
        let engine = Engine {
            config,
            network,
            graphics,
            shared_state,
            local_state,
        };
        Ok(engine)
    }

    fn init(&mut self) -> GameResult {
        match self.config.network_mode() {
            // Server own game shared shared state, so init it
            crate::NetWorkMode::Server => self.shared_state.init()?,
            // Client initialize its shared state when received from server
            crate::NetWorkMode::Client => {}
        };

        if let Err(error) = self.network.init() {
            return Err(GameError::CustomError(error.to_string()));
        }

        Ok(())
    }

    fn tick(&mut self, ctx: &mut Context) {
        match self.config.network_mode() {
            crate::NetWorkMode::Server => self.tick_as_server(ctx),
            crate::NetWorkMode::Client => self.tick_as_client(ctx),
        }
    }
}

impl event::EventHandler<ggez::GameError> for Engine {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, self.config.target_fps()) {
            // First thing to do is to initialize the shared state.
            if self.local_state.is_first_frame() {
                self.init()?;
            }

            // Execute "each frame" code
            self.tick(ctx);

            // Increment the frame counter
            self.local_state.increment_frame_i();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        self.graphics.clear(ctx);

        // Draw entire scene
        self.generate_map_sprites(self.local_state.draw_decor)?;
        self.generate_entities_sprites()?;

        let scene_draw_param = graphics::DrawParam::new()
            .dest(self.local_state.display_scene_offset)
            .scale(self.local_state.display_scene_scale);
        self.graphics
            .draw_scene(ctx, self.local_state.draw_decor, scene_draw_param)?;

        // Draw ui
        let mut mesh_builder = MeshBuilder::new();
        self.generate_menu_sprites()?;
        self.generate_selection_meshes(&mut mesh_builder)?;
        self.generate_debug_meshes(&mut mesh_builder)?;

        let ui_draw_param = graphics::DrawParam::new();
        self.graphics.draw_ui(ctx, ui_draw_param, mesh_builder)?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        let messages = self.collect_mouse_motion(ctx, x, y, dx, dy);
        self.react(messages);
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let messages = self.collect_mouse_down(ctx, button, x, y);
        self.react(messages);
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let messages = self.collect_mouse_up(ctx, button, x, y);
        self.react(messages);
    }
}
