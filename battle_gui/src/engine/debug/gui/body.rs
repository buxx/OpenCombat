use ggegui::egui::{Context as EguiContext, Ui};
use ggez::Context;

use crate::{engine::message::EngineMessage, engine::Engine};

use super::Panel;

impl Engine {
    pub fn debug_gui_body(
        &mut self,
        ctx: &mut Context,
        egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.debug_gui.panel, Panel::Terrain, "Terrain");
            ui.selectable_value(&mut self.debug_gui.panel, Panel::Soldiers, "Soldiers");
            ui.selectable_value(&mut self.debug_gui.panel, Panel::BattleState, "BattleState");
            ui.selectable_value(&mut self.debug_gui.panel, Panel::GuiState, "GuiState");
            ui.selectable_value(
                &mut self.debug_gui.panel,
                Panel::ServerConfig,
                "ServerConfig",
            );
            ui.selectable_value(&mut self.debug_gui.panel, Panel::GuiConfig, "GuiConfig");
            ui.selectable_value(
                &mut self.debug_gui.panel,
                Panel::VisibilityConfig,
                "VisibilityConfig",
            );
            ui.selectable_value(&mut self.debug_gui.panel, Panel::FightConfig, "FightConfig");
            ui.selectable_value(&mut self.debug_gui.panel, Panel::Textures, "Textures");
        });
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.debug_gui.panel, Panel::Explosives, "Explosives");
        });
        ui.separator();

        match self.debug_gui.panel {
            Panel::Terrain => {
                messages.extend(self.debug_gui_terrain(ctx, egui_ctx, ui));
            }
            Panel::Soldiers => {
                messages.extend(self.debug_gui_soldiers(ctx, egui_ctx, ui));
            }
            Panel::BattleState => {
                messages.extend(self.debug_gui_shared_state(ctx, egui_ctx, ui));
            }
            Panel::GuiState => {
                messages.extend(self.debug_gui_gui_state(ctx, egui_ctx, ui));
            }
            Panel::ServerConfig => {
                messages.extend(self.debug_gui_server_config(ctx, egui_ctx, ui));
            }
            Panel::GuiConfig => {
                messages.extend(self.debug_gui_gui_config(ctx, egui_ctx, ui));
            }
            Panel::VisibilityConfig => {
                messages.extend(self.debug_gui_visibility_config(ctx, egui_ctx, ui));
            }
            Panel::FightConfig => {
                messages.extend(self.debug_gui_fight_config(ctx, egui_ctx, ui));
            }
            Panel::Textures => {
                messages.extend(self.debug_gui_textures(ctx, egui_ctx, ui));
            }
            Panel::Explosives => {
                messages.extend(self.debug_gui_explosives(ctx, egui_ctx, ui));
            }
        }

        messages
    }
}
