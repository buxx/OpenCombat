use battle_core::{game::Side, state::battle::message::BattleStateMessage};
use oc_core::morale::Morale;

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn tick_morale(&self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_morale");

        if self.battle_state.frame_i() % self.config.morale_update_freq() == 0 {
            let a_total = self
                .battle_state
                .soldiers()
                .iter()
                .filter(|s| s.side() == &Side::A)
                .count();
            let b_total = self
                .battle_state
                .soldiers()
                .iter()
                .filter(|s| s.side() == &Side::B)
                .count();
            let a_left = self
                .battle_state
                .soldiers()
                .iter()
                .filter(|s| s.side() == &Side::A)
                .filter(|s| s.can_be_count_for_morale())
                .count();
            let b_left = self
                .battle_state
                .soldiers()
                .iter()
                .filter(|s| s.side() == &Side::B)
                .filter(|s| s.can_be_count_for_morale())
                .count();
            let a_morale = Morale(a_left as f32 / a_total as f32);
            let b_morale = Morale(b_left as f32 / b_total as f32);

            return vec![
                RunnerMessage::BattleState(BattleStateMessage::SetAMorale(a_morale)),
                RunnerMessage::BattleState(BattleStateMessage::SetBMorale(b_morale)),
            ];
        }

        vec![]
    }
}
