use serde::{Deserialize, Serialize};

use battle_core::{
    game::Side,
    state::{battle::message::BattleStateMessage, client::ClientStateMessage},
};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RunnerMessage {
    // Messages which modify server state (and can be shared with clients)
    BattleState(BattleStateMessage),
    // Messages to directly send to clients
    ClientsState(ClientStateMessage),
    ClientState(Side, ClientStateMessage),
}
