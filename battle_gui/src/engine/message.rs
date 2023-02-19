use serde_derive::{Deserialize, Serialize};

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
    graphics::message::GraphicsMessage,
};

use super::{event::UIEvent, input::Control};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EngineMessage {
    BattleState(BattleStateMessage), // These messages will be sent to server
    GuiState(GuiStateMessage),
    Graphics(GraphicsMessage),
    PlaySound(Sound),
    ChangeServerConfig(ChangeConfigMessage),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GuiStateMessage {
    SetCursorPoint(WindowPoint),
    SetLeftClickDown(Option<WindowPoint>),
    SetCurrentCursorVector(Option<(WindowPoint, WindowPoint)>),
    ApplyOnSceneDisplayOffset(Offset),
    SetSelectedSquads(Option<SoldierIndex>, Vec<SquadUuid>),
    SetSquadMenu(Option<(WindowPoint, SquadUuid)>),
    SetPendingOrder(Option<PendingOrder>),
    AddCachePointToPendingOrder(WorldPoint),
    SetDisplayPaths(Vec<(WorldPaths, SquadUuid)>),
    PushDebugPoint(DebugPoint),
    SetDebugTerrain(DebugTerrain),
    SetDebugPhysics(DebugPhysics),
    PushUIEvent(UIEvent),
    ChangeSide,
    ScaleUpdate(f32),
    SetControl(Control),
    SetDebugGuiHovered(bool),
    SetDisplayDebugGui(bool),
}
