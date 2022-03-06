use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Message {
    Foo,
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
}

impl Message {
    /// True if message must be send to clients by server.
    pub fn broadcast(&self) -> bool {
        match self {
            Message::Entity(_, _) => true,
            Message::Network(_) => false,
            Message::Foo => false,
        }
    }

    /// True if message must be send to server by clients.
    pub fn sync(&self) -> bool {
        match self {
            Message::Entity(_, _) => false,
            Message::Network(_) => true,
            Message::Foo => false,
        }
    }
}
