use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Grid, Slider, Ui};

use crate::{
    behavior::feeling::UNDER_FIRE_MAX, engine::Engine, entity::soldier::WeaponClass,
    message::Message, types::SoldierIndex,
};

impl Engine {
    pub fn debug_gui_soldiers(
        &mut self,
        ctx: &mut Context,
        egui_ctx: &EguiContext,
        ui: &mut Ui,
    ) -> Vec<Message> {
        let selected_squad = self.local_state.selected_squads();
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
            self.shared_state
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
    ) -> Vec<Message> {
        let soldier = &mut self.shared_state.soldier_mut(*soldier_index);
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
                    "{}x{} ({}x{})",
                    soldier.get_world_point().x.floor(),
                    soldier.get_world_point().y.floor(),
                    self.map
                        .grid_point_from_world_point(&soldier.get_world_point())
                        .x,
                    self.map
                        .grid_point_from_world_point(&soldier.get_world_point())
                        .y
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
                        soldier.set_last_shoot_frame_i(self.local_state.get_frame_i())
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
