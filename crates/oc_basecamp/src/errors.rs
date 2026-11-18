use eframe::egui;

pub fn show(ctx: &egui::Context, open: &mut bool, errors: &[String]) -> bool {
    let mut clear = false;

    egui::Window::new("Errors")
        .open(open)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for error in errors {
                        ui.colored_label(ui.visuals().error_fg_color, error);
                    }
                });
            ui.separator();
            if ui.button("Clear errors").clicked() {
                clear = true;
            }
        });

    clear
}
