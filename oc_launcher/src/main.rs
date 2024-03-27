#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::Path;

use anyhow::{Context, Result};
use eframe::{
    egui::{self, RichText, TextStyle},
    epaint::{Color32, Vec2},
};

use run::BattleLauncher;

mod run;

const EGUI_SCALE: f32 = 2.0;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 700.0)),
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
    map1_preview: Option<egui::TextureHandle>,
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

            ui.horizontal(|ui|{
                let texture: &egui::TextureHandle = self.map1_preview.get_or_insert_with(|| {
                    ui.ctx().load_texture(
                        "Demo2Preview.png",
                        load_image_from_path(Path::new("resources/maps/Demo2/Demo2Preview.png")).unwrap(),
                        Default::default()
                    )
                });
                ui.add(egui::Image::new(texture, Vec2::new(160., 160.)));

                ui.vertical(|ui|{
                    if ui.button("Attack from West").clicked() {
                        self.error = None;
                        if let Err(error) = self.launch_attack_from_west().context("Launch 'attack from north est'") {
                            self.error = Some(format!("{:#}", error))
                        }
                    };
                    if ui.button("Attack from North-Est").clicked() {
                        self.error = None;
                        if let Err(error) = self.launch_attack_from_north_est().context("Launch 'attack from north est'") {
                            self.error = Some(format!("{:#}", error))
                        }
                    };
                });
            });
        });
    }
}

impl Launcher {
    fn launch_attack_from_west(&self) -> Result<()> {
        self.launch(
            "Demo2",
            "assets/demo2_deployment.json",
            vec!["W", "NW", "SW"],
            vec!["ALL"],
        )?;
        Ok(())
    }
    fn launch_attack_from_north_est(&self) -> Result<()> {
        self.launch(
            "Demo2",
            "assets/demo2_deployment2.json",
            vec!["S", "SE", "E"],
            vec!["ALL"],
        )?;
        Ok(())
    }

    fn launch(
        &self,
        map_name: &str,
        deployment: &str,
        side_a_controls: Vec<&str>,
        side_b_controls: Vec<&str>,
    ) -> Result<()> {
        BattleLauncher::new(map_name, &Path::new(deployment).to_path_buf(), "a")?
            .side_a_controls(side_a_controls.into_iter().map(String::from).collect())
            .side_b_controls(side_b_controls.into_iter().map(String::from).collect())
            .launch()?;
        Ok(())
    }
}

fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::io::Reader::open(path)?.decode()?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}
