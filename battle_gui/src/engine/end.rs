use battle_core::state::battle::phase::Phase;
use ggegui::egui::{Align, Align2, Layout, Vec2, Window};
use ggez::{Context, GameResult};

use super::{gui::EGUI_SCALE, message::EngineMessage, Engine};

impl Engine {
    pub fn update_end_gui(&mut self, ctx: &mut Context) -> GameResult<()> {
        let messages = self.end_gui(ctx);
        self.react(messages, ctx)?;

        Ok(())
    }

    pub fn end_gui(&mut self, ctx: &mut Context) -> Vec<EngineMessage> {
        if let Phase::End(victorious, end_reason) = self.battle_state.phase() {
            let drawable_size = ctx.gfx.drawable_size();
            self.egui_backend
                .input
                .set_scale_factor(EGUI_SCALE, drawable_size);
            let egui_ctx = self.egui_backend.ctx();
            let mut messages = vec![];

            let winner = victorious.to_string();
            let reason = end_reason.to_string();

            Window::new("End of battle")
                .collapsible(false)
                .resizable(false)
                .anchor(Align2::CENTER_CENTER, Vec2::new(0., 0.))
                .show(&egui_ctx, |ui| {
                    ui.label(format!(
                        "Battle is end : {} winning by {} victory.",
                        winner, reason
                    ));
                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        if ui.button("Quit").clicked() {
                            messages.push(EngineMessage::Exit)
                        }
                    })
                });

            self.egui_backend.update(ctx);
            return messages;
        }

        vec![]
    }
}
