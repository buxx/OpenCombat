use std::time::Duration;

use eframe::egui;

use oc_game::Game;

use crate::{game, map_view, page::Page};

use super::PageContext;

/// Delay between two checks of battle gui process
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Wait battle gui process exits
pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some(process) = ctx.battle_process.as_mut() else {
        ui.label("No battle in progress");
        return Some(Page::ChooseSquads);
    };

    match process.try_wait() {
        Ok(None) => {
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Battle in progress ...");
                });
            });
            ui.ctx().request_repaint_after(POLL_INTERVAL);
            None
        }
        Ok(Some(status)) => {
            *ctx.battle_process = None;
            if status.success() {
                update_deployments(ui, ctx);
                Some(Page::ChooseSquads)
            } else {
                ctx.errors
                    .push(format!("Battle exited with error ({status})"));
                *ctx.show_errors = true;
                Some(Page::ChooseSquads)
            }
        }
        Err(error) => {
            *ctx.battle_process = None;
            ctx.errors.push(format!("Wait battle: {error}"));
            *ctx.show_errors = true;
            Some(Page::ChooseSquads)
        }
    }
}

fn update_deployments(ui: &egui::Ui, ctx: &mut PageContext) {
    let Some(mut current) = ctx.current_game.take() else {
        return;
    };

    let Game::Battle(battle_game) = &current;
    let battle = ctx
        .battles
        .iter()
        .find(|battle| battle.name() == battle_game.battle_name);
    let mod_ = ctx.mods.iter().find(|m| m.name == battle_game.mod_name);
    let result = match (battle, mod_) {
        (Some(battle), Some(mod_)) => game::after_battle(ctx.app_dir, &mut current, battle, mod_),
        (None, _) => Err(format!("Battle '{}' not found", battle_game.battle_name)),
        (_, None) => Err(format!("Mod '{}' not found", battle_game.mod_name)),
    };
    if let Err(error) = result {
        ctx.errors
            .push(format!("Update game after battle: {error}"));
        *ctx.show_errors = true;
    }

    // Minimap file changed (spawn zones): it must be reloaded
    map_view::forget(ui.ctx(), &game::minimap_path(ctx.app_dir, &current));

    // Keep games list up to date (modification date) and sorted
    if let Some(listed) = ctx.games.iter_mut().find(|g| g.id() == current.id()) {
        *listed = current.clone();
    }
    ctx.games
        .sort_by_key(|Game::Battle(game)| std::cmp::Reverse(game.modified));

    // Deployments changed: reopen the game to reload them (and reset slots)
    *ctx.deployment = None;
    *ctx.squad_slots = vec![];
    let reopened = match ctx.selected() {
        Some((_, scenario)) => game::open(ctx, scenario, current),
        None => {
            *ctx.current_game = Some(current);
            Err("No scenario selected".to_string())
        }
    };
    if let Err(error) = reopened {
        ctx.errors
            .push(format!("Reopen game after battle: {error}"));
        *ctx.show_errors = true;
    }
}
