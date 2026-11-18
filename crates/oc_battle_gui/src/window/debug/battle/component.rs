use std::fmt::Display;

use bevy::prelude::*;
use bevy_egui::prelude::*;
use oc_geo::region::RegionXy;
use oc_individual::IndividualIndex;
use oc_mod::Mod;
use oc_projectile::ProjectileId;
use oc_root::{WcfgInto, WorldConfig, geo::WorldVec2};
use strum_macros::EnumIter;

use crate::{
    ingame::camera::{GoToPoint, region::Region},
    window::debug::subject::Subject,
};

#[derive(Debug, Clone, EnumIter, Default)]
pub enum View {
    #[default]
    None,
    Regions,
    Individuals,
    Projectiles,
    ComponentDetails,
}

impl super::Context {
    pub fn ui_components(
        &mut self,
        w: &WorldConfig,
        ui: &mut egui_dock::egui::Ui,
        commands: &mut Commands,
        _mod_: &Mod,
        world: &crate::world::World,
        ingame: &crate::ingame::state::State,
    ) {
        ui.horizontal(|ui| {
            if ui.button("x").clicked() {
                self.component_view = View::None;
            }
            if ui
                .button(format!("Regions ({})", self.regions.len()))
                .clicked()
            {
                self.component_view = View::Regions;
            };
            if ui
                .button(format!("Individuals ({})", self.individuals.len()))
                .clicked()
            {
                self.component_view = View::Individuals;
            };
            if ui
                .button(format!("Projectiles ({})", self.projectiles.len()))
                .clicked()
            {
                self.component_view = View::Projectiles;
            };
        });

        if let Some(action) = match self.component_view {
            View::None => None,
            View::Regions => self.ui_regions(w, ui, &self.regions),
            View::Individuals => self.ui_subjects(w, ui, &self.individuals, world, ingame),
            View::Projectiles => self.ui_subjects(w, ui, &self.projectiles, world, ingame),
            View::ComponentDetails => self.ui_component_details(w, ui, world, ingame),
        } {
            match action {
                Action::GoToPoint(point) => {
                    commands.trigger(GoToPoint(point));
                }
                Action::ViewComponentDetails(details) => {
                    self.component_view = View::ComponentDetails;
                    self.component_details = Some(details);
                }
            }
        }
    }

    fn ui_regions(
        &self,
        w: &WorldConfig,
        ui: &mut egui::Ui,
        regions: &Vec<Region>,
    ) -> Option<Action> {
        ui.vertical(|ui| {
            for region in regions {
                let region: RegionXy = region.0.into_(w);
                ui.label(format!("{}.{}", region.0.0, region.0.1));
            }
        });
        None
    }

    fn ui_subjects<I: Display>(
        &self,
        _: &WorldConfig,
        ui: &mut egui::Ui,
        subjects: &Vec<Subject<I>>,
        _world: &crate::world::World,
        ingame: &crate::ingame::state::State,
    ) -> Option<Action> {
        let mut action = None;
        ui.vertical(|ui| {
            for subject in subjects {
                let selected = match subject.details {
                    ComponentDetails::Individual(i) => {
                        let squads_individuals = ingame.selected_squads_individuals();
                        if squads_individuals.is_empty() {
                            false
                        } else {
                            let individuals = ingame.selected_individuals();
                            if !individuals.is_empty() {
                                individuals.contains(&i)
                            } else {
                                squads_individuals.contains(&i)
                            }
                        }
                    }
                    ComponentDetails::Projectile(_) => false,
                };

                ui.horizontal(|ui| {
                    let position = subject.physics.position;
                    let region = subject.physics.region;

                    if ui.button("⏵").clicked() {
                        let point = subject.physics.position.into();
                        action = Some(Action::GoToPoint(point));
                    }
                    if ui.button("*").clicked() {
                        action = Some(Action::ViewComponentDetails(subject.details()));
                    }
                    let infos = subject
                        .infos
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let text = egui::RichText::new(format!(
                        "{} {}.{} ({}.{}) {}",
                        subject.i, position.x, position.y, region.0.0, region.0.1, infos
                    ));

                    let text = if selected { text.strong() } else { text };

                    ui.label(text);
                });
            }
        });

        action
    }

    fn ui_component_details(
        &self,
        w: &WorldConfig,
        ui: &mut egui::Ui,
        world: &crate::world::World,
        ingame: &crate::ingame::state::State,
    ) -> Option<Action> {
        match self.component_details {
            Some(details) => match details {
                ComponentDetails::Individual(individual) => {
                    self.ui_component_details_individual(w, ui, individual, world, ingame)
                }
                ComponentDetails::Projectile(projectile) => {
                    self.ui_component_details_projectile(w, ui, projectile, world, ingame)
                }
            },
            None => None,
        }
    }

    fn ui_component_details_individual(
        &self,
        _w: &WorldConfig,
        ui: &mut egui::Ui,
        i: IndividualIndex,
        world: &crate::world::World,
        _ingame: &crate::ingame::state::State,
    ) -> Option<Action> {
        let individual = world.individual(i)?;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(egui::Label::new(format!("{individual:#?}")).wrap());
        });

        None
    }

    fn ui_component_details_projectile(
        &self,
        _w: &WorldConfig,
        _ui: &mut egui::Ui,
        _i: ProjectileId,
        _world: &crate::world::World,
        _ingame: &crate::ingame::state::State,
    ) -> Option<Action> {
        None
    }
}

pub enum Action {
    GoToPoint(WorldVec2),
    ViewComponentDetails(ComponentDetails),
}

#[derive(Debug, Clone, Copy)]
pub enum ComponentDetails {
    Individual(IndividualIndex),
    Projectile(ProjectileId),
}
