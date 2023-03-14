use battle_core::types::{OrderMarkerIndex, SquadUuid, WindowPoint, WorldPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum UIEvent {
    FinishedCursorVector(WindowPoint, WindowPoint), // From, To
    FinishedCursorLeftClick(WindowPoint),
    FinishedCursorRightClick(WindowPoint),
    CursorMove(WindowPoint),
    ImmobileCursorSince(u64),
    DrawPathFinding(SquadUuid, Option<OrderMarkerIndex>, Vec<WorldPoint>),
    DropSquadTo(SquadUuid, WorldPoint),
}
