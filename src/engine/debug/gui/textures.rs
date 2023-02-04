use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Grid, Ui};

use crate::{
    engine::Engine,
    message::{GraphicsMessage, Message},
};

impl Engine {
    pub fn debug_gui_textures(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        let mut messages = vec![];

        if ui.button("Reload files").clicked() {
            messages.push(Message::Graphics(GraphicsMessage::ReloadAll))
        }

        Grid::new(&format!("textures"))
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                let soldiers_files = self.graphics.soldiers_files().clone();
                ui.label("Soldiers");
                ui.horizontal(|ui| {
                    for resource in soldiers_files {
                        if ui
                            .radio_value(
                                self.graphics.soldiers_file_mut(),
                                resource.clone(),
                                resource,
                            )
                            .changed()
                        {
                            messages.push(Message::Graphics(GraphicsMessage::ReloadSoldiersAsset));
                        };
                    }
                });
                ui.end_row();

                let vehicles_files = self.graphics.vehicles_files().clone();
                ui.label("Vehicles");
                ui.horizontal(|ui| {
                    for resource in vehicles_files {
                        if ui
                            .radio_value(
                                self.graphics.vehicles_file_mut(),
                                resource.clone(),
                                resource,
                            )
                            .changed()
                        {
                            messages.push(Message::Graphics(GraphicsMessage::ReloadVehiclesAsset));
                        };
                    }
                });
                ui.end_row();

                let explosions_files = self.graphics.explosions_files().clone();
                ui.label("Explosions");
                ui.horizontal(|ui| {
                    for resource in explosions_files {
                        if ui
                            .radio_value(
                                self.graphics.explosions_file_mut(),
                                resource.clone(),
                                resource,
                            )
                            .changed()
                        {
                            messages
                                .push(Message::Graphics(GraphicsMessage::ReloadExplosionsAsset));
                        };
                    }
                });
                ui.end_row();

                let ui_files = self.graphics.ui_files().clone();
                ui.label("Ui");
                ui.horizontal(|ui| {
                    for resource in ui_files {
                        if ui
                            .radio_value(self.graphics.ui_file_mut(), resource.clone(), resource)
                            .changed()
                        {
                            messages.push(Message::Graphics(GraphicsMessage::ReloadUiAsset));
                        };
                    }
                });
                ui.end_row();
            });

        messages
    }
}
