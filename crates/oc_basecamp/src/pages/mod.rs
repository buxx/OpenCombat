use std::{path::Path, process::Child};

use eframe::egui;

use oc_deployment::deployment::Deployment;
use oc_game::Game;
use oc_root::side::Side;

pub mod battle;
pub mod battle_finished;
pub mod battle_in_progress;
pub mod campaign;
pub mod choose_squads;
pub mod home;
pub mod multiplayer;
pub mod new_battle;
pub mod prepare_battle;
pub mod solo;

use crate::{
    battle::{Battle, Scenario},
    map::Map,
    mods::ModInfo,
    page::Page,
};

pub struct PageContext<'a> {
    pub app_dir: &'a Path,
    pub battles: &'a [Battle],
    pub maps: &'a [Map],
    pub selected_battle: &'a mut Option<usize>,
    pub selected_scenario: &'a mut Option<usize>,
    pub games: &'a mut Vec<Game>,
    pub current_game: &'a mut Option<Game>,
    pub errors: &'a mut Vec<String>,
    pub show_errors: &'a mut bool,
    pub show_all_battles: &'a mut bool,
    pub mods: &'a [ModInfo],
    pub selected_mod: &'a str,
    pub selected_side: &'a mut Option<Side>,
    pub deployment: &'a mut Option<Deployment>,
    pub squad_slots: &'a mut Vec<Option<usize>>,
    pub battle_process: &'a mut Option<Child>,
    pub battle_args: &'a [String],
}

impl<'a> PageContext<'a> {
    pub fn selected_mod_info(&self) -> Option<&'a ModInfo> {
        let mods: &'a [ModInfo] = self.mods;
        mods.iter().find(|m| m.name == self.selected_mod)
    }

    pub fn selected(&self) -> Option<(&'a Battle, &'a Scenario)> {
        let battles: &'a [Battle] = self.battles;
        let battle = battles.get((*self.selected_battle)?)?;
        let scenario = battle.scenarios().get((*self.selected_scenario)?)?;
        Some((battle, scenario))
    }
}

pub fn button_row(ui: &mut egui::Ui, items: [(&str, Page); 2]) -> Option<Page> {
    let mut clicked = None;
    let spacing = ui.spacing().item_spacing;
    let avail = ui.available_size();
    let size = egui::vec2(((avail.x - spacing.x) / 2.0).max(50.0), avail.y.min(200.0));

    ui.horizontal(|ui| {
        for (label, page) in items {
            let button = egui::Button::new(egui::RichText::new(label).size(28.0));
            if ui.add_sized(size, button).clicked() {
                clicked = Some(page);
            }
        }
    });

    clicked
}

pub fn not_implemented(ui: &mut egui::Ui) -> Option<Page> {
    ui.label("not yet implemented");
    None
}
