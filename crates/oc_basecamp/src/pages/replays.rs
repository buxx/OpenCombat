use eframe::egui;

use crate::{page::Page, replays};

use super::PageContext;

pub fn show(ui: &mut egui::Ui, ctx: &mut PageContext) -> Option<Page> {
    if ctx.records.is_none() || ui.button("Refresh").clicked() {
        let (records, errors) = replays::load_all(ctx.app_dir);
        ctx.errors.extend(errors);
        *ctx.records = Some(records);
    }
    let records = ctx.records.as_deref().unwrap_or_default();

    ui.separator();

    if records.is_empty() {
        ui.label("No replay yet: check \"Record for replay\" before starting a battle.");
        return None;
    }

    let mut play = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for record in records {
            let button = egui::Button::selectable(false, record.name());
            if ui
                .add(button)
                .on_hover_text(record.path.display().to_string())
                .clicked()
            {
                play = Some(record);
            }
        }
    });

    if let Some(record) = play {
        match replays::play(record, ctx.battle_args) {
            Ok(child) => ctx.replay_processes.push(child),
            Err(error) => {
                ctx.errors.push(format!("Play replay: {error}"));
                *ctx.show_errors = true;
            }
        }
    }

    None
}
