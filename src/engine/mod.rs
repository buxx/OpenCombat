use ggez::event::{KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, MeshBuilder};
use ggez::timer::check_update_time;
use ggez::{event, GameError};
use ggez::{Context, GameResult};

use crate::audio::player::Player;
use crate::config::Config;
use crate::graphics::Graphics;
use crate::map::Map;
use crate::network::Network;
use crate::state::local::LocalState;
use crate::state::shared::SharedState;
mod animate;
mod behavior;
mod client;
mod debug;
mod draw;
pub mod input;
mod interior;
mod movement;
mod network;
mod order;
mod physics;
mod react;
mod server;
mod side;
mod soldier;
mod ui;
mod update;
mod utils;
mod vehicle;
mod visibility;

pub struct Engine {
    config: Config,
    network: Network,
    graphics: Graphics,
    player: Option<Player>,
    map: Map,
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
        local_state: LocalState,
        map: Map,
    ) -> GameResult<Engine> {
        let network = Network::new(config.clone())?;
        let engine = Engine {
            config,
            network,
            graphics,
            player: None,
            map,
            shared_state,
            local_state,
        };
        Ok(engine)
    }

    fn init(&mut self, ctx: &mut Context) -> GameResult {
        match self.config.network_mode() {
            // Server own game shared shared state, so init it
            crate::NetWorkMode::Server => {
                self.shared_state.init()?;
                self.graphics.initialize(self.shared_state.soldiers());
            }
            // Client initialize its shared state when received from server
            crate::NetWorkMode::Client => {}
        };

        if let Err(error) = self.network.init() {
            return Err(GameError::CustomError(error.to_string()));
        }

        self.player = Some(Player::new(ctx)?);

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
                self.init(ctx)?;
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
        self.generate_soldiers_sprites()?;
        self.generate_vehicles_sprites()?;

        let scene_draw_param = graphics::DrawParam::new()
            .dest(self.local_state.display_scene_offset.to_vec2())
            .scale(self.local_state.display_scene_scale.to_vec2());
        self.graphics
            .draw_scene(ctx, self.local_state.draw_decor, scene_draw_param)?;

        // Draw ui
        let mut mesh_builder = MeshBuilder::new();
        self.generate_menu_sprites()?;

        self.draw_debug_terrain(ctx, scene_draw_param)?;
        self.draw_physics(&mut mesh_builder)?;
        self.generate_debug_meshes(&mut mesh_builder)?;
        self.generate_selection_meshes(&mut mesh_builder)?;
        self.generate_display_paths_meshes(&mut mesh_builder)?;
        self.generate_game_play_meshes(&mut mesh_builder)?;
        self.generate_orders_sprites()?;

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

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        let messages = self.collect_mouse_wheel(ctx, x, y);
        self.react(messages);
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymods: KeyMods,
        repeat: bool,
    ) {
        let messages = self.collect_key_pressed(ctx, keycode, keymods, repeat);
        self.react(messages);
    }

    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        let messages = self.collect_key_released(ctx, keycode);
        self.react(messages);
    }
}
