use crate::{behavior::Behavior, sync::StateCopy, types::*};
use serde::{Deserialize, Serialize};

pub mod result;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Message {
    State(StateMessage),
    Network(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum StateMessage {
    Entity(EntityIndex, EntityMessage),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EntityMessage {
    SetWorldPosition(WorldPosition),
    SetBehavior(Behavior),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum NetworkMessage {
    Acknowledge,
    RequireCompleteSync,
    InitializeStateFrom(StateCopy),
}

impl Message {
    pub fn vec_from_entity(i: EntityIndex, messages: Vec<EntityMessage>) -> Vec<Message> {
        messages
            .into_iter()
            .map(|m| Message::State(StateMessage::Entity(i, m)))
            .collect()
    }
}
