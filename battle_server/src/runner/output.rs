use battle_core::{game::Side, message::OutputMessage};

use super::{message::RunnerMessage, Runner, RunnerError};

impl Runner {
    pub fn outputs(&self, messages: &Vec<RunnerMessage>) -> Result<(), RunnerError> {
        let mut outputs = vec![];

        for message in messages {
            // WARNING : Be careful here. Here, messages are send to client for replication.
            // Change what is send or not will change gui battle state
            match message {
                // TODO : We can filter message where content is really a change ?
                RunnerMessage::BattleState(message) => {
                    //
                    outputs.push((Side::All, OutputMessage::BattleState(message.clone())))
                }
                RunnerMessage::ClientsState(client_state_message) => {
                    //
                    outputs.push((
                        Side::All,
                        OutputMessage::ClientState(client_state_message.clone()),
                    ))
                }
                RunnerMessage::ClientState(side, client_state_message) => {
                    //
                    outputs.push((
                        side.clone(),
                        OutputMessage::ClientState(client_state_message.clone()),
                    ))
                }
            }
        }

        self.send(outputs)?;

        Ok(())
    }

    fn send(&self, outputs: Vec<(Side, OutputMessage)>) -> Result<(), RunnerError> {
        // TODO : send to correct side (for now, all is send to all)
        let messages = outputs.iter().map(|o| o.1.clone()).collect();
        match self.output.send(messages) {
            Ok(_) => Ok(()),
            Err(error) => Result::Err(RunnerError::Output(error)),
        }
    }
}
