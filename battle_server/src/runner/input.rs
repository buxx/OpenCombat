use battle_core::message::{InputMessage, OutputMessage};
use crossbeam_channel::TryRecvError;

use super::{Runner, RunnerError};

impl Runner {
    pub fn inputs(&mut self) -> Result<(), RunnerError> {
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
                        println!("RequireCompleteSync!");
                        self.output
                            .send(vec![OutputMessage::LoadFromCopy(self.battle_state.copy())])
                            .expect("FIXME From chelou");
                    }
                    InputMessage::BattleState(battle_state_message) => {
                        side_effects
                            .extend(self.battle_state.react(&battle_state_message, self.frame_i));
                    }
                    InputMessage::ChangeConfig(change_config) => {
                        self.output
                            .send(vec![OutputMessage::ChangeConfig(change_config.clone())])
                            .expect("FIXME From chelou");
                        self.config.react(&change_config);
                    }
                };
            }
        }

        Ok(())
    }
}
