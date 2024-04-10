use battle_core::types::{SoldierIndex, SquadUuid, WorldPoint};

#[derive(Debug, Clone)]
pub enum HudEvent {
    RequestBeginBattle,
    RequestEndBattle,
    SelectSquad(SquadUuid),
    CenterMapOnSquad(SquadUuid),
    SelectSoldier(SoldierIndex),
    CenterMapOn(WorldPoint),
}
