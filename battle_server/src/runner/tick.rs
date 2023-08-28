use battle_core::state::battle::message::BattleStateMessage;

use crate::runner::message::RunnerMessage;

use super::{Runner, RunnerError};

impl Runner {
    pub fn tick(&mut self) -> Result<(), RunnerError> {
        let frame_i = self.battle_state.frame_i();
        puffin::profile_scope!("tick", format!("frame {frame_i}"));
        self.inputs()?;

        let mut messages = vec![RunnerMessage::BattleState(
            BattleStateMessage::IncrementFrameI,
        )];
        messages.extend(self.tick_phase());
        messages.extend(self.tick_morale());
        messages.extend(self.tick_victory());
        messages.extend(self.tick_flags());
        messages.extend(self.tick_soldiers());
        messages.extend(self.tick_update_squad_leaders());
        messages.extend(self.tick_feeling_decreasing_soldiers());
        messages.extend(self.tick_visibilities());
        messages.extend(self.tick_physics());
        self.react(&messages);
        self.clean();

        self.outputs(&messages)?;
        Ok(())
    }

    pub fn clean(&mut self) {
        self.battle_state.clean();
    }
}
