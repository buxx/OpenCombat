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
                            .react(&battle_state_message, self.gui_state.frame_i()),
                    );
                    match self
                        .output
                        .send(vec![InputMessage::BattleState(battle_state_message)])
                    {
                        Ok(_) => {}
                        Err(error) => {
                            // FIXME : stop here is a good idea ?
                            return Err(GameError::CustomError(format!(
                                "Error when try to send data to server : {}",
                                error
                            )));
                        }
                    }
                }
                EngineMessage::GuiState(gui_state_message) => {
                    //
                    self.gui_state.react(&gui_state_message, ctx)
                }
                EngineMessage::PlaySound(sound) => self.player.play(&sound, ctx)?,
                EngineMessage::Graphics(graphics_message) => self.graphics.react(
                    graphics_message,
                    self.battle_state.map(),
                    &self.server_config,
                    ctx,
                )?,
                EngineMessage::ChangeServerConfig(change_config_message) => {
                    match self
                        .output
                        .send(vec![InputMessage::ChangeConfig(change_config_message)])
                    {
                        Err(error) => {
                            println!("Error when transmit change config message : {}", error)
                        }
                        _ => {}
                    };
                }
                // TODO : manage failures in user display
                EngineMessage::MakeASave => {
                    //
                    match self.save_battle_state() {
                        Ok(save) => self.gui_state.saves_mut().push(save),
                        Err(error) => {
                            eprintln!("Error happen during save : {}", error)
                        }
                    }
                }
                // TODO : manage failures in user display
                EngineMessage::LoadFromSave(save_path) => {
                    if let Some(copy) = self.load_from_save(&save_path) {
                        if let Err(error) =
                            self.output.send(vec![InputMessage::SetBattleState(copy)])
                        {
                            eprintln!(
                                "Error when try to send battle state copy to server : {}",
                                error
                            )
                        }
                    }
                }
                // TODO : manage failures in user display
                EngineMessage::TryLoadLastSave => {
                    let mut saves = self.gui_state.saves().clone();
                    saves.sort();
                    if let Some(save_path) = saves.first() {
                        if let Some(copy) = self.load_from_save(&save_path) {
                            if let Err(error) =
                                self.output.send(vec![InputMessage::SetBattleState(copy)])
                            {
                                eprintln!(
                                    "Error when try to send battle state copy to server : {}",
                                    error
                                )
                            }
                        }
                    }
                }
                EngineMessage::UpdateInteriors => {
                    self.update_interior_sprites();
                }
                EngineMessage::SwitchDecorDisplay => {
                    self.gui_state.draw_decor = !self.gui_state.draw_decor
                }
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
