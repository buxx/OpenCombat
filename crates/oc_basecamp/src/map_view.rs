use std::path::Path;

use eframe::egui;

const MAX_SIZE: f32 = 700.;

pub fn button(ui: &mut egui::Ui, id_salt: &str, path: &Path) {
    let open_id = egui::Id::new(("map_view_open", id_salt));
    let mut open = ui.data(|d| d.get_temp::<bool>(open_id)).unwrap_or(false);

    if ui.button("View map").clicked() {
        open = true;
    }

    if open {
        egui::Window::new("Map")
            .id(open_id)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                ui.add(egui::Image::new(uri(path)).max_size(egui::Vec2::splat(MAX_SIZE)));
            });
    }

    ui.data_mut(|d| d.insert_temp(open_id, open));
}

fn uri(path: &Path) -> String {
    format!("file://{}", path.display())
}

pub fn forget(ctx: &egui::Context, path: &Path) {
    ctx.forget_image(&uri(path));
}
