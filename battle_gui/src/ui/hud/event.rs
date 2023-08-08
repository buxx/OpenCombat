use battle_core::types::SquadUuid;

#[derive(Debug, Clone)]
pub enum HudEvent {
    RequestBeginBattle,
    RequestEndBattle,
    SelectSquad(SquadUuid),
}
