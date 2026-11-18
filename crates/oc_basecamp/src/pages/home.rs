use eframe::egui;

use crate::page::Page;

pub fn show(ui: &mut egui::Ui) -> Option<Page> {
    super::button_row(
        ui,
        [("Solo", Page::Solo), ("Multiplayer", Page::Multiplayer)],
    )
}
