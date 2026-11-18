use std::time::Instant;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui_dock::{DockArea, DockState, Style, TabIndex};
use oc_mod::{Mod, weapons::WeaponType};
use oc_root::WorldConfig;
use strum::IntoEnumIterator;

use crate::ingame::{
    camera::{debug::tile::ToggleShowTiles, squad::ToggleShowFormationPositions},
    debug::visibility::ToggleShowVisibility,
    input::left_click::{LeftClickMode, LeftClickModeType, SetLeftClick},
    lov::SpawnLovConfig,
};

#[derive(Clone)]
pub struct Window {
    pub tree: DockState<super::Tab>,
    pub last: Instant,
    pub context: super::Context,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            tree: DockState::new(super::Tab::iter().collect()),
            last: Instant::now(),
            context: Default::default(),
        }
    }
}

impl Window {
    pub fn show(
        &mut self,
        contexts: &mut EguiContexts,
        commands: &mut Commands,
        mod_: &Mod,
        w: &WorldConfig,
        world: &crate::world::World,
        ingame: &crate::ingame::state::State,
    ) -> Result {
        let ctx = contexts.ctx_mut()?;
        let size = bevy_egui::egui::vec2(600.0, 400.0);
        let mut context =
            super::InContext::new(&mut self.context, commands, mod_, w, world, ingame);
        let mut shortcut = None;

        bevy_egui::egui::Window::new("Dock window")
            .default_size(size)
            .pivot(bevy_egui::egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let show_tiles = &mut context.context.show_tiles;
                    let show_formation_positions = &mut context.context.show_formation_positions;
                    let show_visibilities = &mut context.context.show_visibilities;
                    let mut speed = w.speed();

                    bevy_egui::egui::ComboBox::from_label("Refresh every")
                        .selected_text(format!("{:?}", context.context.refresh))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut context.context.refresh,
                                super::refresh::Refresh::EachFrame,
                                "frame",
                            );
                            ui.selectable_value(
                                &mut context.context.refresh,
                                super::refresh::Refresh::X100ms,
                                "100ms",
                            );
                            ui.selectable_value(
                                &mut context.context.refresh,
                                super::refresh::Refresh::X1s,
                                "1s",
                            );
                        });

                    ui.separator();

                    if ui.checkbox(show_tiles, "Tiles").changed() {
                        context.commands.trigger(ToggleShowTiles);
                    }
                    if ui
                        .checkbox(show_formation_positions, "Formations")
                        .changed()
                    {
                        context.commands.trigger(ToggleShowFormationPositions);
                    }
                    if ui.checkbox(show_visibilities, "Visibility").changed() {
                        context.commands.trigger(ToggleShowVisibility);
                    }

                    egui::ComboBox::from_label("Speed")
                        .selected_text(speed.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut speed, 0.1, "0.1");
                            ui.selectable_value(&mut speed, 0.5, "0.5");
                            ui.selectable_value(&mut speed, 1.0, "1.0");
                            ui.selectable_value(&mut speed, 2.0, "2.0");
                            ui.selectable_value(&mut speed, 5.0, "5.0");
                            ui.selectable_value(&mut speed, 10.0, "10.0");
                        });

                    WorldConfig::set_speed((speed * 10.) as u32);

                    ui.separator();

                    if ui.button("f1").clicked() {
                        shortcut = Some(Shortcut::FireWithFirstRifle);
                    }

                    if ui.button("f2").clicked() {
                        shortcut = Some(Shortcut::LineOfView);
                    }
                });

                DockArea::new(&mut self.tree)
                    .style(Style::from_egui(ctx.style_of(ctx.theme()).as_ref()))
                    .show_inside(ui, &mut context);
            });

        if let Some(shortcut) = shortcut {
            match shortcut {
                Shortcut::FireWithFirstRifle => {
                    self.focus_tab("Left click");
                    self.context.left_click_mode = LeftClickModeType::SpawnProjectile;
                    self.context.spawn_weapon_type = WeaponType::Rifle;
                    self.context.spawn_weapon = mod_
                        .weapons
                        .iter()
                        .find(|weapon| weapon.is_type(WeaponType::Rifle))
                        .cloned();
                    self.context.spawn_ammunition = mod_
                        .ammunitions
                        .iter()
                        .find(|ammunition| {
                            self.context
                                .spawn_weapon
                                .as_ref()
                                .map(|weapon| weapon.ammunitions().clone())
                                .unwrap_or_default()
                                .contains(ammunition)
                        })
                        .cloned();
                    self.context.spawn_shot = self
                        .context
                        .spawn_weapon
                        .as_ref()
                        .and_then(|weapon| weapon.shots().first().cloned());
                }
                Shortcut::LineOfView => {
                    self.focus_tab("Left click");
                    self.context.left_click_mode = LeftClickModeType::LineOfView;

                    let profile = SpawnLovConfig::default();
                    let lov = LeftClickMode::LineOfView(profile);
                    commands.trigger(SetLeftClick(lov));
                }
            }
        }

        Ok(())
    }

    // Warning: method generated by IA
    // TODO: egui_dock is maybe not good enough to be maintained
    fn focus_tab(&mut self, tab_name: &str) {
        let found = self
            .tree
            .iter_all_tabs()
            .find(|(_, t)| t.to_string() == tab_name)
            .map(|(path, _)| path); // path is now a TabPath, not a (SurfaceIndex, NodeIndex) tuple

        if let Some(path) = found
            && let Some(node_tabs) = self.tree[path.surface][path.node].tabs()
            && let Some(tab_index) = node_tabs.iter().position(|t| t.to_string() == tab_name)
        {
            let _ = self.tree.set_active_tab(egui_dock::TabPath {
                surface: path.surface,
                node: path.node,
                tab: TabIndex(tab_index),
            });
        }
    }
}

pub enum Shortcut {
    FireWithFirstRifle,
    LineOfView,
}
