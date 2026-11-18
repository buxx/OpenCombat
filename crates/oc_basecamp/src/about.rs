use eframe::egui;

/// Show the About popin. `open` is set to false when closed.
pub fn show(ctx: &egui::Context, open: &mut bool) {
    egui::Window::new("About")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label(
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do \
                 eiusmod tempor incididunt ut labore et dolore magna aliqua.",
            );
            ui.add_space(8.0);
            ui.hyperlink_to("Lorem ipsum", "https://github.com/buxx/OpenCombat");
        });
}
