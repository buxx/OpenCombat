use std::{path::PathBuf, process::Child};

use eframe::egui;
use oc_deployment::deployment::Deployment;
use oc_game::Game;
use oc_root::{DEFAULT_MOD, side::Side};

use crate::{
    about,
    battle::{self, Battle},
    errors,
    map::{self, Map},
    minimap,
    mods::{self, ModInfo},
    page::Page,
    pages::PageContext,
};

pub struct App {
    /// Application folder in user home (games, cache)
    app_dir: PathBuf,
    loading: bool,
    page: Page,
    mods: Vec<ModInfo>,
    selected_mod: String,
    show_about: bool,
    battles: Vec<Battle>,
    maps: Vec<Map>,
    games: Vec<Game>,
    current_game: Option<Game>,
    selected_battle: Option<usize>,
    selected_scenario: Option<usize>,
    show_all_battles: bool,
    selected_side: Option<Side>,
    deployment: Option<Deployment>,
    squad_slots: Vec<Option<usize>>,
    battle_process: Option<Child>,
    /// Extra arguments given to battle gui (like debug ones)
    battle_args: Vec<String>,
    errors: Vec<String>,
    show_errors: bool,
}

impl App {
    /// Fails if the application folder can't be determined
    pub fn new() -> Result<Self, oc_game::Error> {
        Ok(Self::with_app_dir(oc_game::app_dir()?))
    }

    /// Build the application using `app_dir` as application folder (games, cache).
    pub fn with_app_dir(app_dir: PathBuf) -> Self {
        let (mods, mut errors) = mods::list();
        if !mods.iter().any(|m| m.name == DEFAULT_MOD) {
            errors.push(format!("Default mod '{DEFAULT_MOD}' not found"));
        }

        Self {
            app_dir,
            page: Page::Home,
            mods,
            selected_mod: DEFAULT_MOD.to_string(),
            show_about: false,
            battles: vec![],
            maps: vec![],
            games: vec![],
            current_game: None,
            selected_battle: None,
            selected_scenario: None,
            show_all_battles: false,
            selected_side: None,
            deployment: None,
            squad_slots: vec![],
            battle_process: None,
            battle_args: vec![],
            loading: true,
            errors,
            show_errors: false,
        }
    }

    pub fn with_battle_args(mut self, args: Vec<String>) -> Self {
        self.battle_args = args;
        self
    }

    pub fn page(&self) -> Page {
        self.page
    }

    pub fn is_loading(&self) -> bool {
        self.loading
    }

    pub fn error_messages(&self) -> &[String] {
        &self.errors
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("Other", |ui| {
                if ui.button("About").clicked() {
                    self.show_about = true;
                    ui.close();
                }
            });
        });
    }

    fn navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.errors(ui);
            self.breadcrumbs(ui);
            self.mods(ui);
        });
    }

    fn mods(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let selected_text = self.selected_mod.clone();

            // Mod can only be changed from the Home page
            ui.add_enabled_ui(self.page == Page::Home, |ui| {
                egui::ComboBox::from_id_salt("mod_select")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for mod_ in &self.mods {
                            let value = mod_.name.clone();
                            ui.selectable_value(&mut self.selected_mod, value, &mod_.name);
                        }
                    });
            });

            ui.label("Mod");
        });
    }

    fn breadcrumbs(&mut self, ui: &mut egui::Ui) {
        let path = self.page.path();
        let last = path.len() - 1;
        for (i, page) in path.iter().enumerate() {
            if i == last {
                ui.strong(page.title());
            } else if self.page == Page::BattleInProgress {
                ui.label(page.title());
                ui.label(">");
            } else {
                if ui.link(page.title()).clicked() {
                    self.page = *page;
                }
                ui.label(">");
            }
        }
    }

    fn errors(&mut self, ui: &mut egui::Ui) {
        if !self.errors.is_empty() {
            let icon = egui::RichText::new("⚠").color(ui.visuals().error_fg_color);
            if ui.button(icon).on_hover_text("Errors").clicked() {
                self.show_errors = true;
            }
        }
    }

    fn loading(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading...");
                });
            });
        });

        let (battles, errors) = battle::load_all();
        self.battles = battles;
        self.errors.extend(errors);
        minimap::generate_all(&self.app_dir, &self.battles, &self.mods, &mut self.errors);

        let (maps, errors) = map::load_all(&self.mods);
        self.maps = maps;
        self.errors.extend(errors);

        let (games, errors) = oc_game::load_all(&self.app_dir);
        self.games = games;
        self.errors.extend(errors);

        self.loading = false;
        ui.ctx().request_repaint();
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("navigation").show(ui, |ui| self.navigation(ui));
        egui::CentralPanel::default().show(ui, |ui| {
            let mut ctx = PageContext {
                app_dir: &self.app_dir,
                battles: &self.battles,
                maps: &self.maps,
                games: &mut self.games,
                current_game: &mut self.current_game,
                errors: &mut self.errors,
                show_errors: &mut self.show_errors,
                selected_battle: &mut self.selected_battle,
                selected_scenario: &mut self.selected_scenario,
                show_all_battles: &mut self.show_all_battles,
                selected_side: &mut self.selected_side,
                deployment: &mut self.deployment,
                squad_slots: &mut self.squad_slots,
                battle_process: &mut self.battle_process,
                battle_args: &self.battle_args,
                mods: &self.mods,
                selected_mod: &self.selected_mod,
            };
            if let Some(next) = self.page.show(ui, &mut ctx) {
                // New page must be drawn now, not at next user input (like a redirection)
                if next != self.page {
                    ui.ctx().request_repaint();
                }
                self.page = next;
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("menu_bar").show(ui, |ui| self.menu_bar(ui));

        if self.loading {
            self.loading(ui);
        } else {
            self.show(ui);
        }

        if self.show_errors && errors::show(ui.ctx(), &mut self.show_errors, &self.errors) {
            self.errors.clear();
            self.show_errors = false;
        }

        if self.show_about {
            about::show(ui.ctx(), &mut self.show_about);
        }
    }
}
