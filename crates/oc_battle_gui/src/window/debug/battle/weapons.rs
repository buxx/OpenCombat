use bevy::prelude::*;
use bevy_egui::egui;
use oc_mod::Mod;
use oc_root::WorldConfig;

impl super::Context {
    pub fn weapons(
        &mut self,
        _: &WorldConfig,
        ui: &mut egui_dock::egui::Ui,
        _commands: &mut Commands,
        _mod_: &Mod,
        _world: &crate::world::World,
        _ingame: &crate::ingame::state::State,
    ) {
        let mut inaccuracy = WorldConfig::inaccuracy_spread();
        let mut inaccuracy_enabled = WorldConfig::inaccuracy_spread_enabled();

        ui.checkbox(&mut inaccuracy_enabled, "inaccuracy_spread_enabled");

        ui.spacing_mut().slider_width = ui.available_width() - 50.0;
        ui.add(
            egui::Slider::new(&mut inaccuracy, 0.0..=5.0)
                .step_by(0.001)
                .text("Inaccuracy"),
        );

        WorldConfig::set_inaccuracy_spread(inaccuracy);
        WorldConfig::set_inaccuracy_spread_enabled(inaccuracy_enabled);
    }
}
