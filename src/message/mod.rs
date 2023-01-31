use std::collections::HashMap;

use crate::{
    audio::Sound,
    behavior::{gesture::Gesture, Behavior},
    debug::{DebugPhysics, DebugTerrain},
    engine::input::Control,
    entity::soldier::WeaponClass,
    game::explosive::Type as ExplosiveType,
    order::{Order, PendingOrder},
    physics::{
        effect::Effect,
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
    Graphics(GraphicsMessage),
    Network(NetworkMessage),
    Physics(PhysicsMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PhysicsMessage {
    PushBulletFire(BulletFire),
    PushExplosion(Explosion),
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
    PushPhysicsEffect(Effect),
    // TODO : Why this is a shared state message ? It should be only local no ?
    PushSoundToPlay(Sound),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LocalStateMessage {
    SetDebugTerrain(DebugTerrain),
    SetDebugPhysics(DebugPhysics),
    SetCursorPoint(WindowPoint),
    SetLeftClickDown(Option<WindowPoint>),
    SetCurrentCursorVector(Option<(WindowPoint, WindowPoint)>),
    ApplyOnSceneDisplayOffset(Offset),
    SetSelectedSquads(Option<SoldierIndex>, Vec<SquadUuid>),
    SetSquadMenu(Option<(WindowPoint, SquadUuid)>),
    SetPendingOrder(Option<PendingOrder>),
    AddCachePointToPendingOrder(WorldPoint),
    SetDisplayPaths(Vec<(WorldPaths, SquadUuid)>),
    PushUIEvent(UIEvent),
    PushDebugPoint(DebugPoint),
    ChangeSide,
    ScaleUpdate(f32),
    SetControl(Control),
    SetVisibilities(HashMap<(SoldierIndex, SoldierIndex), Visibility>),
    SetDebugGuiHovered(bool),
    SetDisplayDebugGui(bool),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SoldierMessage {
    SetWorldPosition(WorldPoint),
    SetBehavior(Behavior),
    SetGesture(Gesture),
    SetOrder(Order),
    SetOrientation(Angle),
    SetAlive(bool),
    SetUnconscious(bool),
    ReachBehaviorStep,
    IncreaseUnderFire(u32),
    DecreaseUnderFire,
    ReloadWeapon(WeaponClass),
    WeaponShot(WeaponClass),
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SideEffect {
    RefreshEntityAnimation(SoldierIndex),
    SoldierFinishHisBehavior(SoldierIndex),
    PlaySound(Sound),
}
