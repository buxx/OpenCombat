use battle_core::{
    audio::Sound,
    state::{
        battle::{
            message::BattleStateMessage,
            phase::{EndReason, Phase, Victorious},
        },
        client::ClientStateMessage,
    },
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn tick_phase(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_phase");

        match self.battle_state.phase() {
            Phase::Placement => self.tick_placement_phase(),
            Phase::Battle => self.tick_battle_phase(),
            Phase::End(victorious, reason) => self.tick_end_phase(victorious, reason),
        }
    }

    pub fn tick_placement_phase(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_placement_phase");
        if self.battle_state.a_ready() && self.battle_state.b_ready() {
            return vec![
                RunnerMessage::BattleState(BattleStateMessage::SetPhase(Phase::Battle)),
                RunnerMessage::ClientsState(ClientStateMessage::PlayInterfaceSound(
                    Sound::TrumpetLongHall,
                )),
            ];
        }

        // FIXME : This is a hack to simply dev tests or now
        if (self.battle_state.a_ready() && !self.battle_state.b_connected())
            || (self.battle_state.b_ready() && !self.battle_state.a_connected())
        {
            return vec![
                RunnerMessage::BattleState(BattleStateMessage::SetPhase(Phase::Battle)),
                RunnerMessage::ClientsState(ClientStateMessage::PlayInterfaceSound(
                    Sound::TrumpetLongHall,
                )),
                RunnerMessage::ClientsState(ClientStateMessage::BattleStarted),
            ];
        }

        vec![]
    }

    pub fn tick_battle_phase(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_battle_phase");
        vec![]
    }

    pub fn tick_end_phase(
        &self,
        _victorious: &Victorious,
        _reason: &EndReason,
    ) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_battle_phase");
        vec![]
    }
}
