use crate::{behavior::Behavior, order::Order, sync::StateCopy, types::*};
use serde::{Deserialize, Serialize};

pub mod result;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Message {
    SharedState(SharedStateMessage),
    Engine(EngineMessage),
    Network(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SharedStateMessage {
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
    SetOrientation(Angle),
    ReachBehaviorStep,
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
            .map(|m| Message::SharedState(SharedStateMessage::Entity(i, m)))
            .collect()
    }
}
