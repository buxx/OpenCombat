use eframe::egui;

use crate::page::Page;

const REPLAYS_HEIGHT: f32 = 45.0;

pub fn show(ui: &mut egui::Ui) -> Option<Page> {
    let clicked = super::button_row(
        ui,
        [("Solo", Page::Solo), ("Multiplayer", Page::Multiplayer)],
    );

    if let Some(value) = replays(ui) {
        return value;
    }

    clicked
}

fn replays(ui: &mut egui::Ui) -> Option<Option<Page>> {
    let size = egui::vec2(ui.available_width(), REPLAYS_HEIGHT);
    let button = egui::Button::new(egui::RichText::new("Replays").size(20.0));
    if ui.add_sized(size, button).clicked() {
        return Some(Some(Page::Replays));
    }
    None
}
