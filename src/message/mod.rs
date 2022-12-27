use std::collections::HashMap;

use crate::{
    audio::Sound,
    behavior::Behavior,
    debug::{DebugLevel, DebugPhysics, DebugTerrain},
    engine::input::Control,
    game::explosive::Type as ExplosiveType,
    order::{Order, PendingOrder},
    physics::{
        event::{bullet::BulletFire, explosion::Explosion},
        visibility::Visibility,
    },
    sync::StateCopy,
    types::*,
    utils::DebugPoint,
};
use serde::{Deserialize, Serialize};

pub mod result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Message {
    LocalState(LocalStateMessage),
    SharedState(SharedStateMessage),
    Network(NetworkMessage),
    Graphics(GraphicsMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GraphicsMessage {
    PushExplosionAnimation(WorldPoint, ExplosiveType),
    RemoveExplosionAnimation(WorldPoint),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SharedStateMessage {
    Soldier(SoldierIndex, SoldierMessage),
    Vehicle(VehicleIndex, VehicleMessage),
    PushCommandOrder(SquadUuid, Order),
    PushSquadOrder(SoldierIndex, Order),
    RemoveCommandOder(SquadUuid),
    RemoveSquadOder(SoldierIndex),
    PushBulletFire(BulletFire),
    PushExplosion(Explosion),
    PushSoundToPlay(Sound),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LocalStateMessage {
    SetDebugLevel(DebugLevel),
    SetDebugTerrain(DebugTerrain),
    SetDebugPhysics(DebugPhysics),
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
    SetControl(Control),
    SetVisibilities(HashMap<(SoldierIndex, SoldierIndex), Visibility>),
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
    SetChassisOrientation(Angle),
    // SetMainTurretOrientation(Angle),
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
