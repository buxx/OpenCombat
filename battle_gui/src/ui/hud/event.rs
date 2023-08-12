use battle_core::types::{SquadUuid, WorldPoint};

#[derive(Debug, Clone)]
pub enum HudEvent {
    RequestBeginBattle,
    RequestEndBattle,
    SelectSquad(SquadUuid),
    CenterMapOn(WorldPoint),
}
