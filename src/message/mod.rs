use crate::{
    behavior::Behavior,
    debug::{DebugLevel, DebugTerrain},
    engine::input::Control,
    order::{Order, PendingOrder},
    sync::StateCopy,
    types::*,
    utils::DebugPoint,
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
    Soldier(SoldierIndex, SoldierMessage),
    Vehicle(VehicleIndex, VehicleMessage),
    PushCommandOrder(SquadUuid, Order),
    PushSquadOrder(SoldierIndex, Order),
    RemoveCommandOder(SquadUuid),
    RemoveSquadOder(SoldierIndex),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum LocalStateMessage {
    SetDebugLevel(DebugLevel),
    SetDebugTerrain(DebugTerrain),
    SetCursorPoint(WindowPoint),
    SetLeftClickDown(Option<WindowPoint>),
    SetCurrentCursorVector(Option<(WindowPoint, WindowPoint)>),
    ApplyOnSceneDisplayOffset(Offset),
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
    PushDebugPoint(DebugPoint),
    ChangeSide,
    ScaleUpdate(f32),
    AddControl(Control),
    RemoveControl(Control),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SoldierMessage {
    SetWorldPosition(WorldPoint),
    SetBehavior(Behavior),
    SetOrientation(Angle),
    ReachBehaviorStep,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum VehicleMessage {
    SetWorldPosition(WorldPoint),
    SetOrientation(Angle),
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
