use std::sync::atomic::Ordering;

use battle_core::{
    message::OutputMessage,
    state::{
        battle::{message::SideEffect, BattleState},
        client::ClientStateMessage,
    },
    utils::DebugPoint,
};
use ggez::{Context, GameResult};

use super::Engine;

impl Engine {
    pub fn sync(&mut self, ctx: &mut Context) -> GameResult {
        puffin::profile_scope!("sync");
        let mut side_effects = vec![];
        let frame_i = self.gui_state.frame_i();

        while let Ok(messages) = self.input.try_recv() {
            for message in &messages {
                match message {
                    OutputMessage::LoadFromCopy(copy) => {
                        let mut battle_state =
                            BattleState::from_copy(copy, self.battle_state.map());
                        battle_state.resolve();

                        side_effects.extend(
                            battle_state
                                .soldiers()
                                .iter()
                                .map(|soldier| SideEffect::RefreshEntityAnimation(soldier.uuid()))
                                .collect::<Vec<SideEffect>>(),
                        );

                        self.sync_required.swap(false, Ordering::Relaxed);
                        self.battle_state = battle_state;

                        if !self.first_copy_loaded {
                            self.first_copy_loaded = true;
                            self.react(self.when_first_copy_messages.clone(), ctx)?;
                            self.when_first_copy_messages = vec![];
                        }
                    }
                    OutputMessage::BattleState(battle_state_message) => {
                        if self.gui_state.debug_physics_areas {
                            self.inspect_for_bullet_fire_into_debug_points(battle_state_message);
                        }

                        if !self.sync_required.load(Ordering::Relaxed) {
                            side_effects
                                .extend(self.battle_state.react(battle_state_message, frame_i));
                        }
                    }
                    OutputMessage::ClientState(client_state_message) => {
                        match client_state_message {
                            ClientStateMessage::PushDebugPoint(debug_point) => {
                                self.gui_state.debug_points_mut().push(DebugPoint {
                                    frame_i: frame_i + 120,
                                    point: debug_point.point,
                                    color: debug_point.color,
                                })
                            }
                            ClientStateMessage::PlayInterfaceSound(sound) => {
                                self.player.play(sound, ctx)?
                            }
                            ClientStateMessage::PlayBattleSound(sound) => {
                                self.player.play(sound, ctx)?
                            }
                            ClientStateMessage::BattleStarted => {
                                self.graphics.battle_started(ctx, self.battle_state.map())?;
                            }
                        }
                    }
                    OutputMessage::ChangeConfig(change_config) => {
                        self.server_config.react(change_config);
                    }
                }
            }
        }

        self.side_effects(side_effects);

        Ok(())
    }
}
