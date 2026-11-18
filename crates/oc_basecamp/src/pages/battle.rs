use std::path::Path;

use eframe::egui;

use oc_game::Game;
use oc_root::side::Side;

use crate::{
    battle::{Battle, Scenario},
    game,
    map::Map,
    map_view, minimap,
    mods::ModInfo,
    page::Page,
};

use super::PageContext;

const HEADER_HEIGHT: f32 = 28.0;

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    egui::Panel::left("battles_list")
        .resizable(true)
        .default_size(220.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(HEADER_HEIGHT);
                ui.strong("Battle");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(ctx.show_all_battles, "Show all").on_hover_text(
                        "Check to list battles with map which is incompatible with selected mod",
                    );
                });
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, battle) in ctx.battles.iter().enumerate() {
                    if !is_listed(ctx, battle) {
                        continue;
                    }

                    if ui
                        .selectable_label(*ctx.selected_battle == Some(i), battle.name())
                        .clicked()
                    {
                        *ctx.selected_battle = Some(i);
                        *ctx.selected_scenario = None;
                    }
                }
            });
        });

    let mut next = None;
    egui::CentralPanel::default()
        .frame(central_frame(ui))
        .show(ui, |ui| {
            match ctx
                .selected_battle
                .and_then(|i| ctx.battles.get(i))
                .filter(|b| is_listed(ctx, b))
            {
                Some(battle) => {
                    let errors = mod_errors(battle, ctx.maps, ctx.selected_mod);
                    ui.horizontal(|ui| {
                        ui.set_min_height(HEADER_HEIGHT);
                        ui.strong(battle.name());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            compatibility(ui, &errors);
                            map_view::button(
                                ui,
                                "battle",
                                &minimap::battle_path(ctx.app_dir, battle),
                            );
                        });
                    });

                    ui.separator();

                    ui.label(battle.description());

                    ui.add_space(8.0);
                    next = scenarios(
                        ui,
                        ctx.app_dir,
                        battle,
                        ctx.selected_scenario,
                        ctx.games,
                        ctx.selected_mod_info(),
                        ctx.selected_mod,
                    );
                }
                None => {
                    ui.label("Select a battle");
                }
            }
        });

    match next? {
        Action::Go(page) => Some(page),
        Action::Open(game) => open(ctx, game),
    }
}

enum Action {
    Go(Page),
    Open(Game),
}

fn open(ctx: &mut PageContext, game: Game) -> Option<Page> {
    let (_, scenario) = ctx.selected()?;

    match game::open(ctx, scenario, game) {
        Ok(()) => Some(Page::ChooseSquads),
        Err(error) => {
            ctx.errors.push(format!("Open game: {error}"));
            *ctx.show_errors = true;
            None
        }
    }
}

/// Battle is listed if all are shown, or if its map has no error with the selected mod.
fn is_listed(ctx: &PageContext, battle: &Battle) -> bool {
    *ctx.show_all_battles || is_compatible(battle, ctx.maps, ctx.selected_mod)
}

fn is_compatible(battle: &Battle, maps: &[Map], selected_mod: &str) -> bool {
    maps.iter()
        .find(|m| m.name == battle.map())
        .and_then(|map| {
            map.compatibility
                .iter()
                .find(|c| c.mod_name == selected_mod)
        })
        .is_some_and(|compat| compat.is_compatible())
}

/// Errors of the battle map with the selected mod.
fn mod_errors(battle: &Battle, maps: &[Map], selected_mod: &str) -> Vec<String> {
    let Some(map) = maps.iter().find(|m| m.name == battle.map()) else {
        return vec![format!("Map '{}' not found", battle.map())];
    };
    let Some(compat) = map
        .compatibility
        .iter()
        .find(|c| c.mod_name == selected_mod)
    else {
        return vec![format!("Mod '{selected_mod}' not checked")];
    };

    compat.errors.clone()
}

/// Error icon button opening a popin listing errors, when errors.
fn compatibility(ui: &mut egui::Ui, errors: &[String]) {
    if errors.is_empty() {
        return;
    }

    let open_id = egui::Id::new("battle_mod_errors_open");
    let mut open = ui.data(|d| d.get_temp::<bool>(open_id)).unwrap_or(false);

    let icon = egui::RichText::new("⚠")
        .color(ui.visuals().error_fg_color)
        .size(20.0);
    if ui
        .button(icon)
        .on_hover_text("Incompatible with selected mod")
        .clicked()
    {
        open = true;
    }

    if open {
        egui::Window::new("Compatibility errors")
            .open(&mut open)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for error in errors {
                            ui.colored_label(ui.visuals().error_fg_color, error);
                        }
                    });
            });
    }

    ui.data_mut(|d| d.insert_temp(open_id, open));
}

fn scenarios(
    ui: &mut egui::Ui,
    app_dir: &Path,
    battle: &Battle,
    selected: &mut Option<usize>,
    games: &[Game],
    mod_info: Option<&ModInfo>,
    selected_mod: &str,
) -> Option<Action> {
    egui::Panel::left("scenarios_list")
        .resizable(true)
        .default_size(200.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(HEADER_HEIGHT);
                ui.strong("Scenario");
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, scenario) in battle.scenarios().iter().enumerate() {
                    if ui
                        .selectable_label(*selected == Some(i), scenario.name())
                        .clicked()
                    {
                        *selected = Some(i);
                    }
                }
            });
        });

    let mut next = None;
    egui::CentralPanel::default()
        .frame(central_frame(ui))
        .show(ui, |ui| {
            match selected.and_then(|i| battle.scenarios().get(i)) {
                Some(scenario) => {
                    ui.horizontal(|ui| {
                        ui.set_min_height(HEADER_HEIGHT);
                        ui.strong(scenario.name());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let path =
                                minimap::scenario_path(app_dir, battle, scenario, selected_mod);
                            map_view::button(ui, "scenario", &path);
                        });
                    });

                    ui.separator();

                    ui.label(scenario.description());
                    ui.add_space(8.0);

                    next = games_section(ui, battle, scenario, games, mod_info, selected_mod);
                }
                None => {
                    ui.label("Select a scenario");
                }
            }
        });

    next
}

fn games_section(
    ui: &mut egui::Ui,
    battle: &Battle,
    scenario: &Scenario,
    games: &[Game],
    mod_info: Option<&ModInfo>,
    selected_mod: &str,
) -> Option<Action> {
    let mut next = None;

    ui.horizontal(|ui| {
        ui.strong("Games");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("New game").clicked() {
                next = Some(Action::Go(Page::NewBattle));
            }
        });
    });

    ui.separator();

    let scenario_games = games.iter().filter(|Game::Battle(game)| {
        game.mod_name == selected_mod
            && game.battle_name == battle.name()
            && game.scenario_name == scenario.name()
    });

    let mut any = false;
    for game in scenario_games {
        any = true;
        let Game::Battle(battle_game) = game;
        let modified = battle_game.modified.with_timezone(&chrono::Local);
        let side = mod_info
            .map(|m| match battle_game.side {
                Side::A => m.a_name.clone(),
                Side::B => m.b_name.clone(),
            })
            .unwrap_or_else(|| battle_game.side.letter().to_string());
        let label = format!("{} ({side})", modified.format("%Y-%m-%d %H:%M"));
        if ui
            .add(egui::Button::selectable(false, label))
            .on_hover_text("Continue this game")
            .clicked()
        {
            next = Some(Action::Open(game.clone()));
        }
    }
    if !any {
        ui.weak("No games");
    }

    next
}

/// Central panel frame with the same vertical margins as side panels, so headers of both align.
fn central_frame(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::central_panel(ui.style()).inner_margin(egui::Margin::symmetric(8, 2))
}
