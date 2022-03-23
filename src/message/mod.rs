use crate::{
    behavior::Behavior,
    debug::DebugLevel,
    order::{Order, PendingOrder},
    sync::StateCopy,
    types::*,
};
use serde::{Deserialize, Serialize};

pub mod result;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Message {
    LocalState(LocalStateMessage),
    SharedState(SharedStateMessage),
    Network(NetworkMessage),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SharedStateMessage {
    Entity(EntityIndex, EntityMessage),
    PushCommandOrder(SquadUuid, Order),
    PushSquadOrder(EntityIndex, Order),
    RemoveCommandOder(SquadUuid),
    RemoveSquadOder(EntityIndex),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum LocalStateMessage {
    SetDebugLevel(DebugLevel),
    SetCursorPoint(WindowPoint),
    SetLeftClickDown(Option<WindowPoint>),
    SetCurrentCursorVector(Option<(WindowPoint, WindowPoint)>),
    SetSceneDisplayOffset(Offset),
    SetSelectedSquads(Vec<SquadUuid>),
    SetSquadMenu(Option<(WindowPoint, SquadUuid)>),
    SetPendingOrder(
        Option<(
            PendingOrder,
            SquadUuid,
            Option<OrderMarkerIndex>,
            Vec<WorldPoint>,
        )>,
    ),
    AddCachePointToPendingOrder(WorldPoint),
    SetDisplayPaths(Vec<(WorldPaths, SquadUuid)>),
    PushUIEvent(UIEvent),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EntityMessage {
    SetWorldPosition(WorldPoint),
    SetBehavior(Behavior),
    SetOrientation(Angle),
    ReachBehaviorStep,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum UIEvent {
    FinishedCursorVector(WindowPoint, WindowPoint), // From, To
    FinishedCursorLeftClick(WindowPoint),
    FinishedCursorRightClick(WindowPoint),
    CursorMove(WindowPoint),
    ImmobileCursorSince(u64),
    DrawPathFinding(SquadUuid, Option<OrderMarkerIndex>, Vec<WorldPoint>),
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
