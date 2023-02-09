use battle_core::{message::OutputMessage, state::client::ClientStateMessage};
use ggez::{Context, GameResult};

use super::Engine;

impl Engine {
    pub fn sync(&mut self, ctx: &mut Context) -> GameResult {
        puffin::profile_scope!("sync");
        let mut side_effects = vec![];

        while let Ok(messages) = self.input.try_recv() {
            for message in &messages {
                match message {
                    OutputMessage::BattleState(battle_state_message) => {
                        side_effects.extend(
                            self.battle_state
                                .react(battle_state_message, self.gui_state.get_frame_i()),
                        );
                    }
                    OutputMessage::ClientState(client_state_message) => {
                        match client_state_message {
                            ClientStateMessage::PushDebugPoint(debug_point) => {
                                self.gui_state.debug_points_mut().push(debug_point.clone())
                            }
                            ClientStateMessage::PlayInterfaceSound(sound) => {
                                self.player.play(sound, ctx)?
                            }
                            ClientStateMessage::PlayBattleSound(sound) => {
                                self.player.play(sound, ctx)?
                            }
                        }
                    }
                }
            }
        }

        self.side_effects(side_effects);

        Ok(())
    }
}
