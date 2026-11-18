use bevy::prelude::*;
use oc_mod::Mod;
use oc_root::WorldConfig;

impl super::Context {
    pub fn ui_cursor(
        &mut self,
        _: &WorldConfig,
        ui: &mut egui_dock::egui::Ui,
        _commands: &mut Commands,
        _mod_: &Mod,
        _world: &crate::world::World,
        _ingame: &crate::ingame::state::State,
    ) {
        ui.label(format!("cursor: {:?}", self.cursor));
        ui.label(format!("point: {:?}", self.point));
        ui.label(format!("tile: {:?}", self.tile));
        ui.label(format!("region: {:?}", self.region));
    }
}
