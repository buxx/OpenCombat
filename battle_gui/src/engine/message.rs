use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use battle_core::{
    audio::Sound,
    config::ChangeConfigMessage,
    order::PendingOrder,
    state::battle::message::BattleStateMessage,
    types::{Offset, SoldierIndex, SquadUuid, WindowPoint, WorldPaths, WorldPoint},
    utils::DebugPoint,
};

use crate::{
    debug::{DebugPhysics, DebugTerrain},
    graphics::{message::GraphicsMessage, qualified::Zoom},
};

use super::{event::UIEvent, input::Control};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EngineMessage {
    BattleState(BattleStateMessage), // These messages will be sent to server
    GuiState(GuiStateMessage),
    Graphics(GraphicsMessage),
    PlaySound(Sound),
    ChangeServerConfig(ChangeConfigMessage),
    LoadFromSave(PathBuf),
    TryLoadLastSave,
    MakeASave,
    UpdateInteriors,
    SwitchDecorDisplay,
    Exit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GuiStateMessage {
    SetCursorPoint(WindowPoint),
    SetLeftClickDown(Option<WindowPoint>),
    SetCurrentCursorVector(Option<(WindowPoint, WindowPoint)>),
    ApplyOnDisplaySceneOffset(Offset),
    SetDisplaySceneOffset(Offset),
    SetSelectedSquads(Option<SoldierIndex>, Vec<SquadUuid>),
    SetSquadMenu(Option<(WindowPoint, Vec<SquadUuid>)>),
    SetPendingOrders(Vec<PendingOrder>),
    AddCachePointToPendingOrder(WorldPoint),
    SetDisplayPaths(Vec<Vec<(WorldPaths, SquadUuid)>>),
    PushDebugPoint(DebugPoint),
    SetDebugTerrain(DebugTerrain),
    SetDebugPhysics(DebugPhysics),
    PushUIEvent(UIEvent),
    ChangeSide,
    SetZoom(Zoom, WorldPoint),
    SetControl(Control),
    SetDebugGuiHovered(bool),
    SetDisplayDebugGui(bool),
    SetBeginClickOnSoldier(Option<SoldierIndex>),
    SetDragSquad(Option<SquadUuid>),
    SetCursorInHud(bool),
    SetIntroAck(bool),
    SetSavesList(Vec<PathBuf>),
    CenterSceneOn(WorldPoint),
}
