use self::{
    background::Background, battle::BattleButton, detail::SquadDetail, event::HudEvent,
    minimap::Minimap, morale::MoraleIndicator, squad::SquadStatuses,
};
use battle_core::types::WindowPoint;
use ggez::Context;

use super::component::Component;

pub mod background;
pub mod battle;
pub mod builder;
pub mod detail;
pub mod event;
pub mod minimap;
pub mod morale;
pub mod painter;
pub mod squad;

pub const HUD_HEIGHT: f32 = 200.0;

pub struct Hud {
    background: Background,
    battle_button: BattleButton,
    morale_indicator: MoraleIndicator,
    squad_statuses: SquadStatuses,
    squad_detail: SquadDetail,
    minimap: Minimap,
}

impl Hud {
    pub fn new(
        background: Background,
        battle: BattleButton,
        morale_indicator: MoraleIndicator,
        squad_statuses: SquadStatuses,
        squad_detail: SquadDetail,
        minimap: Minimap,
    ) -> Self {
        Self {
            background,
            battle_button: battle,
            morale_indicator,
            squad_statuses,
            squad_detail,
            minimap,
        }
    }

    pub fn background(&self) -> &Background {
        &self.background
    }

    pub fn battle_button(&self) -> &BattleButton {
        &self.battle_button
    }

    pub fn morale_indicator(&self) -> &MoraleIndicator {
        &self.morale_indicator
    }

    pub fn squad_statuses(&self) -> &SquadStatuses {
        &self.squad_statuses
    }

    pub fn squad_detail(&self) -> &SquadDetail {
        &self.squad_detail
    }

    pub fn minimap(&self) -> &Minimap {
        &self.minimap
    }

    pub fn contains(&self, ctx: &Context, points: &[&WindowPoint]) -> bool {
        self.background.contains(ctx, points)
    }

    pub fn hovered_by(
        &self,
        ctx: &Context,
        points: &[&WindowPoint],
    ) -> Option<&dyn Component<HudEvent>> {
        if !self.contains(ctx, points) {
            return None;
        }

        if self.battle_button.contains(ctx, points) {
            return Some(&self.battle_button);
        }

        if self.morale_indicator.contains(ctx, points) {
            return Some(&self.morale_indicator);
        }

        if self.squad_statuses.contains(ctx, points) {
            return Some(&self.squad_statuses);
        }

        if self.minimap.contains(ctx, points) {
            return Some(&self.minimap);
        }

        if self.background.contains(ctx, points) {
            return Some(&self.background);
        }

        None
    }
}
