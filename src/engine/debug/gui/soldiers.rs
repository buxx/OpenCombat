use ggez::Context;
use ggez_egui::egui::{Context as EguiContext, Grid, Slider, Ui};

use crate::{
    behavior::feeling::{Feeling, UNDER_FIRE_MAX},
    engine::Engine,
    message::Message,
    types::SoldierIndex,
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

                println!("{:?}", soldier.under_fire());
            });

        vec![]
    }
}
