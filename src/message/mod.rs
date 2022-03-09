use crate::{behavior::Behavior, order::Order, sync::StateCopy, types::*};
use serde::{Deserialize, Serialize};

pub mod result;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Message {
    State(StateMessage),
    Engine(EngineMessage),
    Network(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum StateMessage {
    Entity(EntityIndex, EntityMessage),
    PushOrder(SquadUuid, Order),
    RemoveOder(SquadUuid),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EngineMessage {
    ApplySceneDisplayOffset(Offset),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EntityMessage {
    SetWorldPosition(WorldPoint),
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
