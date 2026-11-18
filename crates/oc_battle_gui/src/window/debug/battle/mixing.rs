use bevy::prelude::*;
use bevy_egui::egui;

use crate::fx::{SetBusVolume, SetMasterVolume};

impl super::Context {
    pub fn mixing(&mut self, ui: &mut egui_dock::egui::Ui, commands: &mut Commands) {
        ui.spacing_mut().slider_width = ui.available_width() - 150.0;

        let before = self.master_volume;
        ui.add(
            egui::Slider::new(&mut self.master_volume, 0.0..=3.0)
                .step_by(0.05)
                .text("Global"),
        );
        if self.master_volume != before {
            commands.trigger(SetMasterVolume(self.master_volume));
        }
        ui.separator();

        for (bus, volume) in &mut self.bus_volumes {
            let before = *volume;
            ui.add(
                egui::Slider::new(volume, 0.0..=3.0)
                    .step_by(0.05)
                    .text(format!("{bus:?}")),
            );

            if *volume != before {
                commands.trigger(SetBusVolume(*bus, *volume));
            }
        }
    }
}
