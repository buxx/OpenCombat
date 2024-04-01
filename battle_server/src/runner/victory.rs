use battle_core::{
    config::END_MORALE,
    game::{flag::FlagOwnership, Side},
    state::battle::{
        message::BattleStateMessage,
        phase::{EndReason, Phase, Victorious},
    },
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn tick_victory(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_victory");

        if self.battle_state().frame_i() % self.config.victory_update_freq() == 0 {
            // Victory by morale
            if self.battle_state().a_morale().0 <= END_MORALE {
                return vec![RunnerMessage::BattleState(BattleStateMessage::SetPhase(
                    Phase::End(Victorious(Side::B), EndReason::Morale),
                ))];
            }
            if self.battle_state().b_morale().0 <= END_MORALE {
                return vec![RunnerMessage::BattleState(BattleStateMessage::SetPhase(
                    Phase::End(Victorious(Side::A), EndReason::Morale),
                ))];
            }

            // Victory by flags
            if !self.battle_state().flags().ownerships().is_empty() {
                if self
                    .battle_state()
                    .flags()
                    .ownerships()
                    .iter()
                    .all(|(_, o)| o == &FlagOwnership::A)
                {
                    return vec![RunnerMessage::BattleState(BattleStateMessage::SetPhase(
                        Phase::End(Victorious(Side::A), EndReason::Flags),
                    ))];
                }
                if self
                    .battle_state()
                    .flags()
                    .ownerships()
                    .iter()
                    .all(|(_, o)| o == &FlagOwnership::B)
                {
                    return vec![RunnerMessage::BattleState(BattleStateMessage::SetPhase(
                        Phase::End(Victorious(Side::B), EndReason::Flags),
                    ))];
                }
            }
        }

        vec![]
    }
}
