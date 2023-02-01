use ggez::event::MouseButton;
use ggez::graphics::{self, Canvas, Color, MeshBuilder};
use ggez::input::keyboard::KeyInput;
use ggez::{event, GameError};
use ggez::{Context, GameResult};

use crate::audio::player::Player;
use crate::config::Config;
use crate::graphics::Graphics;
use crate::map::Map;
use crate::network::Network;
use crate::state::local::LocalState;
use crate::state::shared::SharedState;
use crate::NetworkMode;

use self::debug::gui::state::DebugGuiState;
mod animate;
mod behavior;
mod client;
mod debug;
mod draw;
mod feeling;
mod fight;
mod gesture;
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
    // Debug gui
    debug_gui: DebugGuiState,
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
            debug_gui: DebugGuiState::default(),
        };
        Ok(engine)
    }

    fn init(&mut self, ctx: &mut Context) -> GameResult {
        match self.config.network_mode() {
            // Server own game shared shared state, so init it
            crate::NetworkMode::Server => {
                self.shared_state.init()?;
                self.graphics.initialize(self.shared_state.soldiers());
            }
            // Client initialize its shared state when received from server
            crate::NetworkMode::Client => {}
        };

        if let Err(error) = self.network.init() {
            return Err(GameError::CustomError(error.to_string()));
        }

        self.player = Some(Player::new(ctx)?);

        Ok(())
    }

    fn tick(&mut self, ctx: &mut Context) {
        match self.config.network_mode() {
            NetworkMode::Server => self.tick_as_server(ctx),
            NetworkMode::Client => self.tick_as_client(ctx),
        }
    }
}

impl event::EventHandler<ggez::GameError> for Engine {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(self.config.target_fps() as u32) {
            // First thing to do is to initialize the shared state.
            if self.local_state.is_first_frame() {
                self.init(ctx)?;
            }

            // Execute "each frame" code
            self.tick(ctx);

            // Increment the frame counter
            self.local_state.increment_frame_i();
        }

        // Debug window
        self.update_debug_gui(ctx);

        self.graphics.tick(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::from((0.392, 0.584, 0.929)));

        self.graphics.clear();
        let scene_draw = graphics::DrawParam::new()
            .dest(self.local_state.display_scene_offset.to_vec2())
            .scale(self.local_state.display_scene_scale.to_vec2());
        let decor = self.local_state.draw_decor;

        // Draw entire scene
        self.generate_map_sprites(self.local_state.draw_decor)?;
        self.generate_soldiers_sprites()?;
        self.generate_vehicles_sprites()?;
        self.generate_explosion_sprites()?;
        self.graphics.draw_scene(&mut canvas, decor, scene_draw)?;

        // Draw ui
        let mut mesh_builder = MeshBuilder::new();
        self.generate_menu_sprites()?;

        self.draw_debug_terrain(ctx, &mut canvas, scene_draw)?;
        self.draw_physics(&mut mesh_builder)?;
        self.generate_debug_meshes(&mut mesh_builder)?;
        self.generate_selection_meshes(&mut mesh_builder)?;
        self.generate_display_paths_meshes(&mut mesh_builder)?;
        self.generate_game_play_meshes(&mut mesh_builder)?;
        self.generate_orders_sprites()?;

        let ui_draw_param = graphics::DrawParam::new();
        self.graphics
            .draw_ui(ctx, &mut canvas, ui_draw_param, mesh_builder)?;

        self.draw_debug_gui(ctx, &mut canvas);

        canvas.finish(ctx)?;

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if !self.local_state.debug_gui_hovered {
            let messages = self.collect_mouse_down(ctx, button, x, y);
            self.react(messages);
        }
        GameResult::Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if !self.local_state.debug_gui_hovered {
            let messages = self.collect_mouse_up(ctx, button, x, y);
            self.react(messages);
        }
        GameResult::Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) -> Result<(), GameError> {
        let messages = self.collect_mouse_motion(ctx, x, y, dx, dy);
        self.react(messages);
        GameResult::Ok(())
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        let messages = self.collect_mouse_wheel(ctx, x, y);
        self.react(messages);
        GameResult::Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> Result<(), GameError> {
        let messages = self.collect_key_pressed(ctx, input);
        self.react(messages);
        GameResult::Ok(())
    }

    fn key_up_event(&mut self, ctx: &mut Context, input: KeyInput) -> Result<(), GameError> {
        let messages = self.collect_key_released(ctx, input);
        self.react(messages);
        GameResult::Ok(())
    }
}
