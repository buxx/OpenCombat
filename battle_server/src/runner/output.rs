use battle_core::{
    game::Side,
    message::OutputMessage,
    state::battle::message::{BattleStateMessage, SoldierMessage, VehicleMessage},
};

use super::{message::RunnerMessage, Runner, RunnerError};

impl Runner {
    pub fn outputs(&self, messages: &Vec<RunnerMessage>) -> Result<(), RunnerError> {
        let mut outputs = vec![];

        for message in messages {
            match message {
                RunnerMessage::BattleState(battle_state_message) => match battle_state_message {
                    BattleStateMessage::Soldier(_, soldier_message) => match soldier_message {
                        // TODO : For some, filter if really changed before send
                        SoldierMessage::SetWorldPosition(_)
                        | SoldierMessage::SetBehavior(_)
                        | SoldierMessage::SetGesture(_)
                        | SoldierMessage::SetOrder(_)
                        | SoldierMessage::SetOrientation(_)
                        | SoldierMessage::SetAlive(_)
                        | SoldierMessage::IncreaseUnderFire(_)
                        | SoldierMessage::DecreaseUnderFire
                        | SoldierMessage::SetLastShootFrameI(_)
                        | SoldierMessage::SetUnconscious(_) => {
                            //
                            outputs.push((
                                Side::All,
                                OutputMessage::BattleState(battle_state_message.clone()),
                            ))
                        }
                        SoldierMessage::WeaponShot(_)
                        | SoldierMessage::ReloadWeapon(_)
                        | SoldierMessage::ReachBehaviorStep => {}
                    },
                    BattleStateMessage::Vehicle(_, vehicle_message) => match vehicle_message {
                        VehicleMessage::SetWorldPosition(_)
                        | VehicleMessage::SetChassisOrientation(_) => outputs.push((
                            Side::All,
                            OutputMessage::BattleState(battle_state_message.clone()),
                        )),
                    },
                    BattleStateMessage::PushBulletFire(_)
                    | BattleStateMessage::PushExplosion(_)
                    | BattleStateMessage::SetVisibilities(_) => outputs.push((
                        Side::All,
                        OutputMessage::BattleState(battle_state_message.clone()),
                    )),
                },
                RunnerMessage::ClientsState(client_state_message) => outputs.push((
                    Side::All,
                    OutputMessage::ClientState(client_state_message.clone()),
                )),
                RunnerMessage::ClientState(side, client_state_message) => outputs.push((
                    side.clone(),
                    OutputMessage::ClientState(client_state_message.clone()),
                )),
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
