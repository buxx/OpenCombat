use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use battle_core::config::{GuiConfig, ServerConfig};
use battle_core::game::Side;
use battle_core::message::{InputMessage, OutputMessage};
use battle_core::state::battle::BattleState;
use crossbeam_channel::{Receiver, Sender};
use ggegui::Gui;
use ggez::event::EventHandler;
use ggez::event::MouseButton;
use ggez::graphics::{self, Canvas, Color, MeshBuilder};
use ggez::input::keyboard::KeyInput;
use ggez::GameError;
use ggez::{Context, GameResult};
use oc_core::spawn::SpawnZoneName;

use crate::audio::player::Player;
use crate::graphics::Graphics;

use self::debug::gui::state::DebugGuiState;
use self::state::GuiState;

mod debug;
mod draw;
mod event;
mod game;
mod input;
mod interior;
mod message;
mod network;
mod order;
mod physics;
mod react;
mod state;
mod tick;
mod ui;
mod utils;

pub struct Engine {
    config: GuiConfig,
    // Mirror of server config used to live debug window
    server_config: ServerConfig,
    graphics: Graphics,
    input: Receiver<Vec<OutputMessage>>,
    output: Sender<Vec<InputMessage>>,
    player: Player,
    /// The current shared state of the game. This struct is own by server and replicated on clients
    battle_state: BattleState,
    /// The current local state of the game.
    gui_state: GuiState,
    sync_required: Arc<AtomicBool>,
    stop_required: Arc<AtomicBool>,
    // Debug gui
    debug_gui: DebugGuiState,
    egui_backend: Gui,
    ///
    a_control: Vec<SpawnZoneName>,
    b_control: Vec<SpawnZoneName>,
}

impl Engine {
    pub fn new(
        ctx: &mut Context,
        side: &Side,
        config: GuiConfig,
        server_config: ServerConfig,
        input_sender: Sender<Vec<InputMessage>>,
        output_receiver: Receiver<Vec<OutputMessage>>,
        graphics: Graphics,
        battle_state: BattleState,
        sync_required: Arc<AtomicBool>,
        stop_required: Arc<AtomicBool>,
        a_control: Vec<SpawnZoneName>,
        b_control: Vec<SpawnZoneName>,
    ) -> GameResult<Engine> {
        let engine = Engine {
            config,
            server_config,
            graphics,
            input: output_receiver, // Gui input is server output
            output: input_sender,   // Gui output is server input
            player: Player::new(ctx)?,
            battle_state,
            gui_state: GuiState::new(side.clone()),
            sync_required,
            stop_required,
            debug_gui: DebugGuiState::new()?,
            egui_backend: Gui::default(),
            a_control,
            b_control,
        };
        Ok(engine)
    }
}

impl EventHandler<ggez::GameError> for Engine {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let frame_i = self.gui_state.get_frame_i();
        puffin::profile_scope!("update", format!("frame {frame_i}"));
        puffin::GlobalProfiler::lock().new_frame();

        while ctx.time.check_update_time(self.config.target_fps) {
            // Execute "each frame" code
            self.tick(ctx)?;

            // Increment the frame counter
            self.gui_state.increment_frame_i();
        }

        self.update_debug_gui(ctx)?;
        self.graphics.tick(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::from((0.392, 0.584, 0.929)));

        self.graphics.clear();
        let scene_draw = graphics::DrawParam::new()
            .dest(self.gui_state.display_scene_offset.to_vec2())
            .scale(self.gui_state.display_scene_scale.to_vec2());
        let decor = self.gui_state.draw_decor;

        // Draw entire scene
        self.generate_map_sprites(self.gui_state.draw_decor)?;
        self.generate_soldiers_sprites()?;
        self.generate_vehicles_sprites()?;
        self.generate_explosion_sprites()?;
        self.graphics.draw_map(&mut canvas, scene_draw)?;
        self.draw_debug_terrain(ctx, &mut canvas, scene_draw)?;
        self.graphics.draw_units(&mut canvas, scene_draw)?;
        self.graphics.draw_decor(&mut canvas, decor, scene_draw)?;

        // Draw ui
        let mut mesh_builder = MeshBuilder::new();
        self.generate_menu_sprites()?;

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
        if !self.gui_state.debug_gui_hovered {
            let messages = self.collect_mouse_down(ctx, button, x, y);
            self.react(messages, ctx)?;
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
        if !self.gui_state.debug_gui_hovered {
            let messages = self.collect_mouse_up(ctx, button, x, y);
            self.react(messages, ctx)?;
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
        self.react(messages, ctx)?;
        GameResult::Ok(())
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, x: f32, y: f32) -> Result<(), GameError> {
        let messages = self.collect_mouse_wheel(ctx, x, y);
        self.react(messages, ctx)?;
        GameResult::Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> Result<(), GameError> {
        let messages = self.collect_key_pressed(ctx, input);
        self.react(messages, ctx)?;
        GameResult::Ok(())
    }

    fn key_up_event(&mut self, ctx: &mut Context, input: KeyInput) -> Result<(), GameError> {
        let messages = self.collect_key_released(ctx, input);
        self.react(messages, ctx)?;
        GameResult::Ok(())
    }

    fn quit_event(&mut self, _ctx: &mut Context) -> Result<bool, ggez::GameError> {
        self.stop_required.store(true, Ordering::Relaxed);
        Ok(false)
    }
}
