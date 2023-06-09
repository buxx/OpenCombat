use battle_core::audio::Sound;
use ggegui::egui::{Align, Align2, Layout, Vec2, Window};
use ggez::{Context, GameResult};

use super::{
    gui::EGUI_SCALE,
    message::{EngineMessage, GuiStateMessage},
    Engine,
};

impl Engine {
    pub fn tick_intro(&self) -> Vec<EngineMessage> {
        if self.gui_state.get_frame_i() == 0 {
            return vec![EngineMessage::PlaySound(Sound::DrumMultiHits)];
        }

        vec![]
    }

    pub fn update_intro_gui(&mut self, ctx: &mut Context) -> GameResult<()> {
        let messages = self.intro_gui(ctx);
        self.react(messages, ctx)?;
        Ok(())
    }

    pub fn intro_gui(&mut self, ctx: &mut Context) -> Vec<EngineMessage> {
        if self.gui_state.intro_ack() {
            return vec![];
        }

        let drawable_size = ctx.gfx.drawable_size();
        self.egui_backend
            .input
            .set_scale_factor(EGUI_SCALE, drawable_size);
        let egui_ctx = self.egui_backend.ctx();
        let mut messages = vec![];

        Window::new("Placement phase")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0., 0.))
            .show(&egui_ctx, |ui| {
                ui.label(concat!(
                    "Before battle, place your squads and prepare they orders. ",
                    "When ready to fight, click on \"Begin\" button.",
                    "\n\n",
                    "F12: Open debug window\n",
                    "F4: Make a save point\n",
                    "F5: Load las save point if any\n",
                    "\n",
                    "CTRL+Click : Move map\n",
                    "T : Hide/show trees\n"
                ));
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if ui.button("Continue").clicked() {
                        messages.push(EngineMessage::GuiState(GuiStateMessage::SetIntroAck(true)))
                    }
                })
            });

        self.egui_backend.update(ctx);
        messages
    }
}
