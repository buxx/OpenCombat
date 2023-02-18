use battle_core::{message::InputMessage, state::battle::message::SideEffect};
use ggez::{Context, GameError, GameResult};

use super::{message::EngineMessage, Engine};

impl Engine {
    pub fn react(&mut self, messages: Vec<EngineMessage>, ctx: &mut Context) -> GameResult {
        puffin::profile_scope!("react");
        let mut side_effects = vec![];

        for message in messages {
            match message {
                EngineMessage::BattleState(battle_state_message) => {
                    // Update gui battle state and modify server battle state to
                    side_effects.extend(
                        self.battle_state
                            .react(&battle_state_message, self.gui_state.get_frame_i()),
                    );
                    match self
                        .output
                        .send(vec![InputMessage::BattleState(battle_state_message)])
                    {
                        Ok(_) => {}
                        Err(error) => {
                            return Err(GameError::CustomError(format!(
                                "Error when try to send data to server : {}",
                                error
                            )))
                        }
                    }
                }
                EngineMessage::GuiState(gui_state_message) => {
                    //
                    self.gui_state.react(&gui_state_message)
                }
                EngineMessage::PlaySound(sound) => self.player.play(&sound, ctx)?,
                EngineMessage::Graphics(graphics_message) => self.graphics.react(
                    graphics_message,
                    self.battle_state.map(),
                    &self.config,
                    ctx,
                )?,
            }
        }

        self.side_effects(side_effects);

        Ok(())
    }

    pub fn side_effects(&mut self, side_effects: Vec<SideEffect>) {
        for side_effect in side_effects {
            match side_effect {
                SideEffect::RefreshEntityAnimation(soldier_index) => {
                    let soldier = self.battle_state.soldier(soldier_index);
                    self.graphics.refresh_soldier_animation(soldier);
                }
                // Server side effect
                SideEffect::SoldierFinishHisBehavior(_) => {}
            }
        }
    }
}
