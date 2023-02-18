use serde_derive::{Deserialize, Serialize};

use crate::{audio::Sound, utils::DebugPoint};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientStateMessage {
    PushDebugPoint(DebugPoint),
    PlayInterfaceSound(Sound),
    // TODO : ajouter le WorldPoint
    PlayBattleSound(Sound),
}
