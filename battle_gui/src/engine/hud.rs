use battle_core::{game::Side, state::battle::message::BattleStateMessage};

use crate::ui::hud::event::HudEvent;

use super::{message::EngineMessage, Engine};

impl Engine {
    pub fn hud_event(&self, event: HudEvent) -> Vec<EngineMessage> {
        match event {
            HudEvent::RequestBeginBattle => self.request_begin_battle(),
            HudEvent::RequestEndBattle => self.request_end_battle(),
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
        vec![]
    }
}
