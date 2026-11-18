use eframe::egui;

use oc_game::{BattleStatus, Game};

use crate::{game, launch, map_view, page::Page};

use super::PageContext;

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    let Some(deployment) = ctx.deployment.as_ref() else {
        ui.label("No deployment loaded");
        return None;
    };

    let Some(current) = ctx.current_game.as_ref() else {
        ui.label("No current game");
        return None;
    };
    let Game::Battle(battle_game) = current;
    if let BattleStatus::Finished(_) = battle_game.status {
        return Some(Page::BattleFinished);
    }
    let fights = battle_game.status.fights();
    let map_path = game::minimap_path(ctx.app_dir, current);

    // Label and soldiers count of each squad
    let squads: Vec<(&str, usize)> = deployment
        .squads
        .iter()
        .map(|squad| (squad.label.as_str(), squad.individuals.len()))
        .collect();

    egui::Panel::left("squads_list")
        .resizable(true)
        .default_size(220.0)
        .show(ui, |ui| {
            map_view::button(ui, "choose_squads", &map_path);

            ui.separator();

            ui.strong("Squads");

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, (label, soldiers)) in squads.iter().enumerate() {
                    let placed = ctx.squad_slots.contains(&Some(index));
                    ui.horizontal(|ui| {
                        let button = egui::Button::selectable(false, *label);
                        let response = ui.add_enabled(!placed, button);
                        ui.label(soldiers_text(*soldiers));
                        if response.clicked()
                            && let Some(slot) = ctx.squad_slots.iter_mut().find(|s| s.is_none())
                        {
                            *slot = Some(index);
                        }
                    });
                }
            });
        });

    let mut start = false;
    egui::Panel::bottom("choose_squads_actions").show(ui, |ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let ready = ctx.squad_slots.iter().any(Option::is_some);
            let button = egui::Button::new(egui::RichText::new("Start battle").size(20.0));
            let response = ui
                .add_enabled(ready, button)
                .on_disabled_hover_text("Choose at least one squad");
            start = response.clicked();
        });
    });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.label(format!("Fights played: {fights}"));
        ui.separator();

        ui.strong("Slots");
        ui.separator();

        for (number, slot) in ctx.squad_slots.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{}.", number + 1));
                match slot.and_then(|index| squads.get(index)) {
                    Some((label, soldiers)) => {
                        ui.label(*label);
                        ui.label(soldiers_text(*soldiers));
                        if ui.button("Remove").clicked() {
                            *slot = None;
                        }
                    }
                    None => {
                        ui.weak("empty");
                    }
                }
            });
        }
    });

    if !start {
        return None;
    }

    match launch::start(ctx) {
        Ok(child) => {
            *ctx.battle_process = Some(child);
            Some(Page::BattleInProgress)
        }
        Err(error) => {
            ctx.errors.push(format!("Start battle: {error}"));
            *ctx.show_errors = true;
            None
        }
    }
}

/// One "o" by soldier
fn soldiers_text(soldiers: usize) -> String {
    "o".repeat(soldiers)
}
