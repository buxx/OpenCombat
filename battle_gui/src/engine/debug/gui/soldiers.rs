use battle_core::{
    behavior::feeling::UNDER_FIRE_MAX, entity::soldier::WeaponClass, types::SoldierIndex,
};
use ggegui::egui::{Context as EguiContext, Grid, Slider, Ui};
use ggez::Context;

use crate::engine::{message::EngineMessage, Engine};

impl Engine {
    pub fn debug_gui_soldiers(
        &mut self,
        ctx: &mut Context,
        egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<EngineMessage> {
        let selected_squad = self.gui_state.selected_squads();
        if selected_squad.1.len() == 0 {
            ui.label("No soldiers selected");
            return vec![];
        } else if selected_squad.1.len() > 1 {
            ui.label("Please select no more than one squad");
            return vec![];
        }

        let soldier_indexes = if let Some(selected_soldier) = selected_squad.0 {
            vec![selected_soldier]
        } else {
            self.battle_state
                .squad(selected_squad.1[0])
                .members()
                .clone()
        };

        let mut messages = vec![];
        for soldier_index in &soldier_indexes {
            messages.extend(self.debug_gui_soldier(ctx, egui_ctx, ui, soldier_index));
            ui.separator();
        }

        messages
    }

    pub fn debug_gui_soldier(
        &mut self,
        _ctx: &mut Context,
        _egui_ctx: &EguiContext,
        ui: &mut Ui,
        soldier_index: &SoldierIndex,
    ) -> Vec<EngineMessage> {
        let soldier = &mut self.battle_state.soldier_mut(*soldier_index);
        // FIXME BS NOW : changes are not sent to server
        Grid::new(&format!("soldier_{}", soldier_index))
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Index");
                ui.label(format!("{}", soldier_index));
                ui.end_row();

                ui.label("Coordinates");
                ui.label(format!(
                    "{}x{}",
                    soldier.world_point().x.floor(),
                    soldier.world_point().y.floor(),
                ));
                ui.end_row();

                ui.label("Alive");
                ui.checkbox(soldier.alive_mut(), "");
                ui.end_row();

                ui.label("Unconscious");
                ui.checkbox(soldier.unconscious_mut(), "");
                ui.end_row();

                ui.label("LastShootFrameI");
                ui.horizontal(|ui| {
                    if ui.button("set").clicked() {
                        soldier.set_last_shoot_frame_i(self.gui_state.frame_i())
                    }
                    ui.label(format!("{}", soldier.last_shoot_frame_i()));
                });
                ui.end_row();

                ui.label("Order");
                ui.label(format!("{}", soldier.order()));
                ui.end_row();

                ui.label("Behavior");
                ui.label(format!("{}", soldier.behavior()));
                ui.end_row();

                ui.label("UnderFire");
                ui.add(Slider::new(
                    soldier.under_fire_mut().value_mut(),
                    0..=UNDER_FIRE_MAX,
                ));
                ui.end_row();

                ui.label("MainWeapon");
                let weapon_text = if let Some(weapon) = soldier.weapon(&WeaponClass::Main) {
                    format!("{}", weapon.name())
                } else {
                    "".to_string()
                };
                ui.label(weapon_text);
                ui.end_row();

                ui.label("Magazines");
                let magazines_text = soldier
                    .magazines()
                    .iter()
                    .map(|magazine| magazine.name())
                    .collect::<Vec<&str>>()
                    .join(", ");
                ui.label(magazines_text);
                ui.end_row();
            });

        vec![]
    }
}
