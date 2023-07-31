use serde::{Deserialize, Serialize};

use crate::{audio::Sound, utils::NewDebugPoint};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientStateMessage {
    PushDebugPoint(NewDebugPoint),
    PlayInterfaceSound(Sound),
    // TODO : ajouter le WorldPoint
    PlayBattleSound(Sound),
    BattleStarted,
}
