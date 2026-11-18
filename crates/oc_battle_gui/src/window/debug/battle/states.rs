use bevy::prelude::*;
use bevy_egui::prelude::*;
use oc_mod::Mod;
use oc_root::WorldConfig;
use strum_macros::EnumIter;

#[derive(Debug, Clone, EnumIter, Default)]
pub enum View {
    #[default]
    Ingame,
}

impl super::Context {
    pub fn ui_states(
        &mut self,
        w: &WorldConfig,
        ui: &mut egui_dock::egui::Ui,
        _commands: &mut Commands,
        _mod_: &Mod,
        _world: &crate::world::World,
        _ingame: &crate::ingame::state::State,
    ) {
        ui.horizontal(|ui| {
            if ui.button("Ingame").clicked() {
                self.states_view = View::Ingame;
            }
        });

        match self.states_view {
            View::Ingame => self.ui_ingame(w, ui),
        }
    }

    fn ui_ingame(&self, _w: &WorldConfig, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // TODO: Auto display state instead hard write it
            ui.label(format!(
                "Selected squads: {:?}",
                self.ingame.selected_squads()
            ));
        });
    }
}
