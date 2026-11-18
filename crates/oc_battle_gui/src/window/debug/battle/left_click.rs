use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::prelude::*;
use oc_mod::{Mod, weapons::WeaponType};
use oc_root::{WorldConfig, side::Side};
use strum::IntoEnumIterator;

use crate::ingame::{
    debug::projectile::SpawnProjectileProfile,
    input::left_click::{LeftClickMode, LeftClickModeType, SetLeftClick},
    lov::SpawnLovConfig,
};

impl super::Context {
    // TODO: seems we can improve this code
    pub fn ui_left_click(
        &mut self,
        _: &WorldConfig,
        ui: &mut egui_dock::egui::Ui,
        commands: &mut Commands,
        mod_: &Mod,
        _world: &crate::world::World,
        _ingame: &crate::ingame::state::State,
    ) {
        let left_click_mode_before = self.left_click_mode;
        let left_click_mode = &mut self.left_click_mode;

        egui::ComboBox::from_label("Left click mode")
            .selected_text(left_click_mode.name())
            .show_ui(ui, |ui| {
                for item in LeftClickModeType::iter() {
                    let item_ = item;
                    let name = item_.name();
                    ui.selectable_value(left_click_mode, item, name);
                }
            });

        match left_click_mode {
            LeftClickModeType::Dragging => {}
            LeftClickModeType::Select => {
                commands.trigger(SetLeftClick(LeftClickMode::Select));
            }
            LeftClickModeType::Order => {
                // TODO: must have OrderType list too
            }
            LeftClickModeType::LineOfView => {
                let click_mode = &mut self.lov_click_mode;
                let click_mode_before = *click_mode;
                let start_plus_z = &mut self.lov_start_plus_z;
                let start_plus_z_before = *start_plus_z;
                let end_plus_z = &mut self.lov_end_plus_z;
                let end_plus_z_before = *end_plus_z;

                egui::ComboBox::new("click_mode", "")
                    .selected_text(click_mode.to_string())
                    .show_ui(ui, |ui| {
                        for item in super::LovClickMode::iter() {
                            let name = item.to_string();
                            ui.selectable_value(click_mode, item, name);
                        }
                    });

                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut start_plus_z.0)
                            .range((0.0)..=5.0)
                            .speed(0.1),
                    );
                    ui.label("+ start z");
                    ui.separator();
                    ui.add(
                        egui::DragValue::new(&mut end_plus_z.0)
                            .range((0.0)..=5.0)
                            .speed(0.1),
                    );
                    ui.label("+ end z");
                });

                if *click_mode != click_mode_before
                    || start_plus_z.0 != start_plus_z_before.0
                    || end_plus_z.0 != end_plus_z_before.0
                {
                    commands.trigger(SetLeftClick(LeftClickMode::LineOfView(SpawnLovConfig {
                        click: *click_mode,
                        start_plus_z: *start_plus_z,
                        stop_plus_z: *end_plus_z,
                    })));
                }
            }
            LeftClickModeType::SpawnProjectile => {
                let weapon_type_before = self.spawn_weapon_type;
                let weapon_type = &mut self.spawn_weapon_type;
                let weapon_before = self.spawn_weapon.clone();
                let weapon = &mut self.spawn_weapon;
                let ammunition = &mut self.spawn_ammunition;
                let shot = &mut self.spawn_shot;
                let repeat = &mut self.spawn_repeat;
                let click_mode = &mut self.spawn_projectile_click_mode;
                let plus_z = &mut self.spawn_projectile_plus_z;
                let side = &mut self.spawn_projectile_side;

                ui.horizontal(|ui| {
                    egui::ComboBox::new("weapon_type", "")
                        .selected_text(weapon_type.name())
                        .show_ui(ui, |ui| {
                            for item in WeaponType::iter() {
                                let name = item.name();
                                ui.selectable_value(weapon_type, item, name);
                            }
                        });

                    egui::ComboBox::new("weapon", "")
                        .selected_text(weapon.as_ref().map(|p| p.name()).unwrap_or_default())
                        .show_ui(ui, |ui| {
                            let items = mod_.weapons.iter().filter(|p| p.is_type(*weapon_type));
                            for item in items {
                                let item = item.clone();
                                let name = item.name();
                                ui.selectable_value(weapon, Some(item.clone()), name);
                            }
                        });

                    if let Some(weapon) = &weapon {
                        egui::ComboBox::new("ammunition", "")
                            .selected_text(
                                ammunition.as_ref().map(|p| p.name()).unwrap_or_default(),
                            )
                            .show_ui(ui, |ui| {
                                for item in weapon.ammunitions() {
                                    let item = item.clone();
                                    let name = item.name();
                                    ui.selectable_value(ammunition, Some(item.clone()), name);
                                }
                            });

                        egui::ComboBox::new("shot_mode", "")
                            .selected_text(shot.as_ref().map(|p| p.name()).unwrap_or_default())
                            .show_ui(ui, |ui| {
                                for item in weapon.shots() {
                                    let item = item.clone();
                                    let name = item.name();
                                    ui.selectable_value(shot, Some(item.clone()), name);
                                }
                            });
                    }

                    egui::ComboBox::new("click_mode", "")
                        .selected_text(click_mode.to_string())
                        .show_ui(ui, |ui| {
                            for item in super::SpawnProjectileClickMode::iter() {
                                let name = item.to_string();
                                ui.selectable_value(click_mode, item, name);
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(repeat).range(1..=255).speed(1));
                    ui.label("count");

                    ui.separator();

                    ui.add(
                        egui::DragValue::new(&mut plus_z.0)
                            .range((0.0)..=5.0)
                            .speed(0.1),
                    );
                    ui.label("+z");

                    ui.separator();

                    egui::ComboBox::new("side", "")
                        .selected_text(side.letter().to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(side, Side::A, "A");
                            ui.selectable_value(side, Side::B, "B");
                        });
                    ui.label("side");
                });

                if weapon_type_before != self.spawn_weapon_type {
                    self.spawn_weapon = None;
                    self.spawn_ammunition = None;
                    self.spawn_shot = None;
                }

                if weapon_before != self.spawn_weapon {
                    self.spawn_ammunition = None;
                    self.spawn_shot = None;
                }

                if let (Some(weapon), Some(ammunition), Some(shot)) =
                    (&self.spawn_weapon, &self.spawn_ammunition, &self.spawn_shot)
                {
                    let spawn = SpawnProjectileProfile::new(
                        weapon.index(),
                        ammunition.index(),
                        shot.index(),
                        *repeat,
                        *plus_z,
                        *side,
                        Duration::ZERO,
                    );
                    commands.trigger(SetLeftClick(LeftClickMode::SpawnProjectile(spawn)));
                }
            }
            LeftClickModeType::Support => {}
            LeftClickModeType::Explosion => {
                let shell = &mut self.explosion_shell;
                let shell_before = *shell;
                let selected = shell
                    .map(|i| mod_.shell(i).name.as_str())
                    .unwrap_or_default();

                egui::ComboBox::new("explosion_shell", "Shell")
                    .selected_text(selected)
                    .show_ui(ui, |ui| {
                        for item in &mod_.shells {
                            ui.selectable_value(shell, Some(item.index()), item.name.as_str());
                        }
                    });

                if let Some(shell) = *shell
                    && (Some(shell) != shell_before
                        || left_click_mode_before != self.left_click_mode)
                {
                    commands.trigger(SetLeftClick(LeftClickMode::Explosion(shell)));
                }
            }
        }

        if left_click_mode_before != self.left_click_mode {
            match self.left_click_mode {
                LeftClickModeType::Select => {
                    commands.trigger(SetLeftClick(LeftClickMode::Select));
                }
                LeftClickModeType::LineOfView => {
                    let profile = SpawnLovConfig::default();
                    let lov = LeftClickMode::LineOfView(profile);
                    commands.trigger(SetLeftClick(lov));
                }
                _ => {}
            }
        }
    }
}
