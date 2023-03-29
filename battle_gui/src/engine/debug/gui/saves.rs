use std::{
    path::PathBuf,
    str::FromStr,
    time::{Duration, SystemTime},
};

use ggez::Context;

use crate::engine::{message::EngineMessage, Engine};
use ggegui::egui::{Context as EguiContext, Grid, Ui};

impl Engine {
    pub fn debug_gui_saves(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        Grid::new("saves")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                if ui.button("Make a save").clicked() {
                    match self.save_battle_state() {
                        Ok(save) => self.gui_state.saves_mut().push(save),
                        Err(error) => {
                            eprintln!("Error happen during save : {}", error)
                        }
                    }
                }
                ui.end_row();

                for save_path in self.gui_state.saves() {
                    if let Some(label) = save_label(save_path) {
                        ui.label(&label);
                        if ui.button("Charger").clicked() {
                            messages.push(EngineMessage::LoadFromSave(save_path.clone()))
                        }
                        ui.end_row()
                    }
                }
            });

        messages
    }
}

fn save_label(save_path: &PathBuf) -> Option<String> {
    if let Some(file_stem) = save_path.file_stem() {
        if let Some(file_stem) = file_stem.to_str() {
            if let Ok(timestamp_s) = u64::from_str(file_stem) {
                let duration = Duration::from_secs(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        - timestamp_s,
                );
                return Some(humantime::format_duration(duration).to_string());
            }
        }
    }

    None
}
