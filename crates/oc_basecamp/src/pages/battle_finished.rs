use eframe::egui;

use oc_game::{BattleResult, BattleStatus, Game};

use crate::page::Page;

use super::PageContext;

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some(Game::Battle(game)) = ctx.current_game.as_ref() else {
        ui.label("No current game");
        return None;
    };

    ui.vertical_centered(|ui| {
        ui.heading("Battle finished");
        if let BattleStatus::Finished(result) = game.status {
            ui.label(match result {
                BattleResult::Victory => "Victory",
                BattleResult::Defeat => "Defeat",
            });
        }
    });

    None
}
