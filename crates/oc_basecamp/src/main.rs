use eframe::egui;

use oc_basecamp::{UI_ZOOM, app, use_exe_dir_if_data_beside};

fn main() -> eframe::Result {
    if let Err(error) = use_exe_dir_if_data_beside() {
        eprintln!("Use executable folder as current folder: {error}");
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 1000.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Open Combat",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_zoom_factor(UI_ZOOM);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            match app::App::new() {
                Ok(app) => Ok(Box::new(app)),
                Err(error) => Ok(Box::new(FatalError(error.to_string()))),
            }
        }),
    )
}

/// Displayed instead of the application when it can't start
struct FatalError(String);

impl eframe::App for FatalError {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Unable to start");
            ui.colored_label(ui.visuals().error_fg_color, &self.0);
        });
    }
}
