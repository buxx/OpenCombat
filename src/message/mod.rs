use crate::{sync::StateCopy, types::*};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Message {
    Entity(EntityIndex, EntityMessage),
    Network(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EntityMessage {
    UpdateWorldPosition(WorldPosition),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum NetworkMessage {
    Acknowledge,
    RequireCompleteSync,
    InitializeStateFrom(StateCopy),
}
