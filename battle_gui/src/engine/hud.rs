use battle_core::{
    game::Side,
    state::battle::{
        message::BattleStateMessage,
        phase::{EndReason, Phase, Victorious},
    },
    types::SquadUuid,
};

use crate::{engine::message::GuiStateMessage, ui::hud::event::HudEvent};

use super::{message::EngineMessage, Engine};

impl Engine {
    pub fn hud_event(&self, event: HudEvent) -> Vec<EngineMessage> {
        match event {
            HudEvent::RequestBeginBattle => self.request_begin_battle(),
            HudEvent::RequestEndBattle => self.request_end_battle(),
            HudEvent::SelectSquad(squad_id) => self.select_squad(&squad_id),
            HudEvent::CenterMapOn(point) => {
                vec![EngineMessage::GuiState(GuiStateMessage::CenterSceneOn(
                    point,
                ))]
            }
        }
    }

    pub fn request_begin_battle(&self) -> Vec<EngineMessage> {
        vec![EngineMessage::BattleState(match self.gui_state.side() {
            Side::A => BattleStateMessage::SetAReady(true),
            Side::B => BattleStateMessage::SetBReady(true),
            _ => panic!("Gui state should never be All"),
        })]
    }

    pub fn request_end_battle(&self) -> Vec<EngineMessage> {
        vec![EngineMessage::BattleState(BattleStateMessage::SetPhase(
            Phase::End(Victorious(Side::All), EndReason::Aborted),
        ))]
    }

    pub fn select_squad(&self, squad_id: &SquadUuid) -> Vec<EngineMessage> {
        vec![EngineMessage::GuiState(GuiStateMessage::SetSelectedSquads(
            Some(self.battle_state.squad(*squad_id).leader()),
            vec![*squad_id],
        ))]
    }
}
