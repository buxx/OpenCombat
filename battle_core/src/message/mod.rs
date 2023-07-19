use serde::{Deserialize, Serialize};

use crate::{
    config::ChangeConfigMessage,
    deployment::Deployment,
    game::control::MapControl,
    state::{battle::message::BattleStateMessage, client::ClientStateMessage},
    sync::BattleStateCopy,
};

use self::network::NetworkMessage;

pub mod network;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    Input(InputMessage),
    Output(OutputMessage),
    Network(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InputMessage {
    LoadDeployment(Deployment),
    LoadControl((MapControl, MapControl)),
    RequireCompleteSync,
    SetBattleState(BattleStateCopy),
    BattleState(BattleStateMessage),
    ChangeConfig(ChangeConfigMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OutputMessage {
    LoadFromCopy(BattleStateCopy),
    BattleState(BattleStateMessage),
    ClientState(ClientStateMessage),
    ChangeConfig(ChangeConfigMessage),
}
