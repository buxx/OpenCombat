use battle_core::{
    message::{InputMessage, OutputMessage},
    state::battle::BattleState,
};
use crossbeam_channel::TryRecvError;

use super::{Runner, RunnerError};

impl Runner {
    pub fn inputs(&mut self) -> Result<(), RunnerError> {
        puffin::profile_scope!("inputs");
        loop {
            let inputs = match self.input.try_recv() {
                Ok(message) => message,
                Err(error) => match error {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => return Err(RunnerError::InputChannelClosed),
                },
            };
            log::debug!("Received {} inputs : {:?}", inputs.len(), &inputs);

            let mut side_effects = vec![];
            for input in inputs {
                match input {
                    InputMessage::RequireCompleteSync => {
                        self.output
                            .send(vec![OutputMessage::LoadFromCopy(self.battle_state.copy())])?;
                    }
                    InputMessage::BattleState(battle_state_message) => {
                        side_effects
                            .extend(self.battle_state.react(&battle_state_message, self.frame_i));
                    }
                    InputMessage::ChangeConfig(change_config) => {
                        self.output
                            .send(vec![OutputMessage::ChangeConfig(change_config.clone())])?;
                        self.config.react(&change_config);
                    }
                    InputMessage::SetBattleState(copy) => {
                        //
                        self.battle_state = BattleState::from_copy(&copy, self.battle_state.map());
                        self.battle_state.resolve();
                        self.output.send(vec![OutputMessage::LoadFromCopy(copy)])?;
                    }
                };
            }
        }

        Ok(())
    }
}
