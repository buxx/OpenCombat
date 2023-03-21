use ggegui::egui::{self, ScrollArea};
use ggez::{Context, GameResult};

use crate::engine::{
    gui::EGUI_SCALE,
    message::{EngineMessage, GuiStateMessage},
    Engine,
};

pub mod body;
pub mod config;
pub mod explosives;
pub mod header;
pub mod meta;
pub mod soldiers;
pub mod state;
pub mod terrain;
pub mod textures;

#[derive(PartialEq, Eq)]
pub enum Panel {
    Terrain,
    Soldiers,
    BattleState,
    GuiState,
    ServerConfig,
    GuiConfig,
    VisibilityConfig,
    FightConfig,
    Textures,
    Explosives,
}

impl Default for Panel {
    fn default() -> Self {
        Self::Terrain
    }
}

impl Engine {
    pub fn update_debug_gui(&mut self, ctx: &mut Context) -> GameResult<()> {
        let messages = self.debug_gui(ctx);
        self.react(messages, ctx)?;
        Ok(())
    }

    pub fn debug_gui(&mut self, ctx: &mut Context) -> Vec<EngineMessage> {
        if !self.gui_state.display_debug_gui() {
            return vec![EngineMessage::GuiState(
                GuiStateMessage::SetDebugGuiHovered(false),
            )];
        }

        let drawable_size = ctx.gfx.drawable_size();
        self.egui_backend
            .input
            .set_scale_factor(EGUI_SCALE, drawable_size);
        let egui_ctx = self.egui_backend.ctx();
        let mut messages = vec![];

        egui::Window::new("Debug").show(&egui_ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                messages.extend(self.debug_gui_header(ctx, &egui_ctx, ui));
                messages.extend(self.debug_gui_body(ctx, &egui_ctx, ui));
            })
        });

        messages.push(EngineMessage::GuiState(
            GuiStateMessage::SetDebugGuiHovered(egui_ctx.is_pointer_over_area()),
        ));

        self.egui_backend.update(ctx);
        messages
    }
}
