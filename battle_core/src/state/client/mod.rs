use serde_derive::{Deserialize, Serialize};

use crate::{audio::Sound, utils::NewDebugPoint};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientStateMessage {
    PushDebugPoint(NewDebugPoint),
    PlayInterfaceSound(Sound),
    // TODO : ajouter le WorldPoint
    PlayBattleSound(Sound),
}
