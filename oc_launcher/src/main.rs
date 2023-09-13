#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::{Context, Result};
use eframe::{
    egui::{self, RichText, TextStyle},
    epaint::Color32,
};

use run::BattleLauncher;

mod run;

const EGUI_SCALE: f32 = 2.0;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<Launcher>::default()),
    )
}

#[derive(Default)]
struct Launcher {
    error: Option<String>,
}

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(EGUI_SCALE);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Open Combat (Proof of Concept)");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Links : ");
                ui.hyperlink_to("📄 Official website", "https://opencombat.bux.fr");
                ui.hyperlink_to("🖮 Discord", "https://discord.gg/6P2vtFh2Px");
                ui.hyperlink_to(" Github", "https://github.com/buxx/OpenCombat");
            });
            ui.separator();

            ui.horizontal_wrapped(|ui| {
                // Trick so we don't have to add spaces in the text below:
                let width = ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                ui.spacing_mut().item_spacing.x = width;

                ui.label("Open Combat is a real-time tactical game which takes place during the 2nd World War.");
                ui.label("Project goal is to include complete combat simulation : Infantry, mortars, vehicles, strategic view, etc.");
            });

            ui.horizontal_wrapped(|ui| {
                // Trick so we don't have to add spaces in the text below:
                let width = ui.fonts(|f| f.glyph_width(&TextStyle::Body.resolve(ui.style()), ' '));
                ui.spacing_mut().item_spacing.x = width;

                ui.label("This demo is an proof of concept. Join us on discord to participate !");
            });

            ui.separator();

            if let Some(error) = &self.error {
                ui.label(RichText::new(error).color(Color32::RED));
            }

            ui.vertical(|ui|{
                if ui.button("foo").clicked() {
                    self.error = None;
                    if let Err(error) = self.launch_attack_from_north_est().context("Launch 'attack from north est'") {
                        self.error = Some(format!("{:#}", error))
                    }
                };
            });
        });
    }
}

impl Launcher {
    fn launch_attack_from_north_est(&self) -> Result<()> {
        BattleLauncher::new()?.launch()?;
        Ok(())
    }
}
