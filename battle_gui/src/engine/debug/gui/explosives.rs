use crate::engine::{message::EngineMessage, Engine};
use battle_core::{config::ChangeConfigMessage, game::explosive::ExplosiveType, types::Distance};
use ggegui::egui::{ComboBox, Context as EguiContext, Grid, Slider, Ui};
use ggez::Context;
use strum::IntoEnumIterator;

impl Engine {
    pub fn debug_gui_explosives(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        let mut messages = vec![];

        ui.horizontal(|ui| {
            ComboBox::from_label("Select explosive to configure")
                .selected_text(format!("{:?}", &mut self.debug_gui.explosive))
                .show_ui(ui, |ui| {
                    ui.style_mut().wrap = Some(false);
                    ui.set_min_width(60.0);
                    for explosive in ExplosiveType::iter() {
                        let text = explosive.to_string();
                        ui.selectable_value(&mut self.debug_gui.explosive, explosive, text);
                    }
                });
        });
        ui.end_row();

        Grid::new("explosives".to_string())
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                if let (
                    Some(direct_death_rayons),
                    Some(regressive_death_rayon),
                    Some(regressive_injured_rayon),
                ) = (
                    self.server_config
                        .explosive_direct_death_rayon
                        .get_mut(&self.debug_gui.explosive),
                    self.server_config
                        .explosive_regressive_death_rayon
                        .get_mut(&self.debug_gui.explosive),
                    self.server_config
                        .explosive_regressive_injured_rayon
                        .get_mut(&self.debug_gui.explosive),
                ) {
                    ui.label("Direct death rayon");
                    // if ui.button("reset").clicked() {
                    //     *value = default;
                    // };
                    if ui
                        .add(Slider::new(
                            &mut direct_death_rayons.millimeters,
                            0..=100000,
                        ))
                        .changed()
                    {
                        messages.push(EngineMessage::ChangeServerConfig(
                            ChangeConfigMessage::ExplosiveDirectDeathRayon(
                                self.debug_gui.explosive.clone(),
                                Distance::from_millimeters(direct_death_rayons.millimeters),
                            ),
                        ));
                    };
                    ui.end_row();

                    ui.label("Regressive death rayon");
                    // if ui.button("reset").clicked() {
                    //     *value = default;
                    // };
                    if ui
                        .add(Slider::new(
                            &mut regressive_death_rayon.millimeters,
                            0..=100000,
                        ))
                        .changed()
                    {
                        messages.push(EngineMessage::ChangeServerConfig(
                            ChangeConfigMessage::ExplosiveRegressiveDeathRayon(
                                self.debug_gui.explosive.clone(),
                                Distance::from_millimeters(regressive_death_rayon.millimeters),
                            ),
                        ));
                    };
                    ui.end_row();

                    ui.label("Regressive injured rayon");
                    // if ui.button("reset").clicked() {
                    //     *value = default;
                    // };
                    if ui
                        .add(Slider::new(
                            &mut regressive_injured_rayon.millimeters,
                            0..=100000,
                        ))
                        .changed()
                    {
                        messages.push(EngineMessage::ChangeServerConfig(
                            ChangeConfigMessage::ExplosiveRegressiveInjuredRayon(
                                self.debug_gui.explosive.clone(),
                                Distance::from_millimeters(regressive_injured_rayon.millimeters),
                            ),
                        ));
                    };
                    ui.end_row();
                }
            });

        messages
    }
}
