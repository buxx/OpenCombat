use battle_core::{
    game::{
        flag::{FlagOwnership, FlagsOwnership},
        Side,
    },
    state::battle::message::BattleStateMessage,
};

use super::{message::RunnerMessage, Runner};

impl Runner {
    pub fn tick_flags(&self) -> Vec<RunnerMessage> {
        if self.battle_state.frame_i() % self.config.soldier_update_freq() == 0 {
            let mut new_ownerships = vec![];
            for (flag_name, ownership) in self.battle_state.flags().ownerships() {
                let flag = self.battle_state.map().flag(flag_name);
                let a_inside = self
                    .battle_state
                    .there_is_side_soldier_in(&Side::A, flag.shape());
                let b_inside = self
                    .battle_state
                    .there_is_side_soldier_in(&Side::B, flag.shape());

                let new_ownership = match (ownership, a_inside, b_inside) {
                    (FlagOwnership::Nobody, true, true) => FlagOwnership::Both,
                    (FlagOwnership::Nobody, true, false) => FlagOwnership::A,
                    (FlagOwnership::Nobody, false, true) => FlagOwnership::B,
                    (FlagOwnership::Nobody, false, false) => FlagOwnership::Nobody,
                    (FlagOwnership::A, true, true) => FlagOwnership::Both,
                    (FlagOwnership::A, true, false) => FlagOwnership::A,
                    (FlagOwnership::A, false, true) => FlagOwnership::B,
                    (FlagOwnership::A, false, false) => FlagOwnership::A,
                    (FlagOwnership::B, true, true) => FlagOwnership::Both,
                    (FlagOwnership::B, true, false) => FlagOwnership::A,
                    (FlagOwnership::B, false, true) => FlagOwnership::B,
                    (FlagOwnership::B, false, false) => FlagOwnership::B,
                    (FlagOwnership::Both, true, true) => FlagOwnership::Both,
                    (FlagOwnership::Both, true, false) => FlagOwnership::A,
                    (FlagOwnership::Both, false, true) => FlagOwnership::B,
                    (FlagOwnership::Both, false, false) => FlagOwnership::Both,
                };
                new_ownerships.push((flag_name.clone(), new_ownership));
            }
            return vec![RunnerMessage::BattleState(
                BattleStateMessage::SetFlagsOwnership(FlagsOwnership::new(new_ownerships)),
            )];
        }

        vec![]
    }
}
