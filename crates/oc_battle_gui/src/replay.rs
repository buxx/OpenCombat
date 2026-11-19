use std::time::Duration;

use bevy::prelude::*;
use bevy_egui::{
    EguiContexts, EguiPrimaryContextPass,
    egui::{self, Align2, Color32, CornerRadius, Sense, Stroke, StrokeKind},
};
use oc_geo::region::WorldRegionIndex;
use oc_root::Wcfg;
use oc_utils::{bevy::EntityMapping, let_some};

use crate::{
    config::Config,
    ingame::{
        individual::ForgotIndividual,
        projectile::ForgotProjectile,
        region::{ForgottenRegion, ListeningRegion},
    },
    network::replay::{Handle, Status},
    states::AppState,
    world::VisibilitiesUpdated,
};

const BAR_HEIGHT: f32 = 14.;
const BAR_WIDTH_RATIO: f32 = 0.6;
const SPEEDS: [f32; 4] = [1., 2., 5., 10.];

#[derive(Resource, Default)]
pub struct Replay(pub Option<Handle>);

#[derive(Debug, Event)]
pub struct ReplayReset;

pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Replay>()
            .add_observer(on_replay_reset)
            .add_systems(
                EguiPrimaryContextPass,
                progress_bar
                    .run_if(in_state(AppState::InGame))
                    .run_if(|replay: Res<Replay>| replay.0.is_some()),
            );
    }
}

fn on_replay_reset(
    _: On<ReplayReset>,
    mut commands: Commands,
    w: Res<Wcfg>,
    individuals: Res<EntityMapping<oc_individual::IndividualIndex>>,
    projectiles: Res<EntityMapping<oc_projectile::ProjectileId>>,
) {
    let_some!(w = &w.0, return);
    tracing::debug!("Replay reset");

    for i in individuals.0.keys() {
        commands.trigger(ForgotIndividual(*i));
    }
    for i in projectiles.0.keys() {
        commands.trigger(ForgotProjectile(*i));
    }
    // Replay listens all regions: forget them (tiles, decor, craters, ...) and listen them again
    for region in (0..w.regions_count()).map(WorldRegionIndex) {
        commands.trigger(ForgottenRegion(region));
        commands.trigger(ListeningRegion(region));
    }
}

fn progress_bar(
    mut contexts: EguiContexts,
    mut commands: Commands,
    replay: Res<Replay>,
    mut config: ResMut<Config>,
) -> Result {
    let_some!(replay = &replay.0, return Ok(()));
    let status = replay.status();
    let ctx = contexts.ctx_mut()?;
    let width = ctx.content_rect().width() * BAR_WIDTH_RATIO;

    egui::Area::new("replay-progress".into())
        .anchor(Align2::CENTER_BOTTOM, [0., -10.])
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    let pause = if status.paused { "Play" } else { "Pause" };
                    if ui.button(pause).clicked() {
                        replay.set_paused(!status.paused);
                    }

                    egui::ComboBox::from_id_salt("replay-speed")
                        .selected_text(speed(status.speed))
                        .width(50.)
                        .show_ui(ui, |ui| {
                            for value in SPEEDS {
                                let selected = status.speed == value;
                                if ui.selectable_label(selected, speed(value)).clicked()
                                    && !selected
                                {
                                    replay.set_speed(value);
                                }
                            }
                        });

                    let end = if status.finished { " (end)" } else { "" };
                    ui.label(format!(
                        "{} / {}{end}",
                        time(status.position),
                        time(status.total)
                    ));

                    // Keep the slot when not jumping: bar must not move
                    if status.seeking {
                        let spinner = egui::Spinner::new().size(BAR_HEIGHT);
                        ui.add_sized([BAR_HEIGHT, BAR_HEIGHT], spinner);
                    } else {
                        ui.add_space(BAR_HEIGHT);
                    }

                    // Checkbox at right: laid out first, then bar takes the remaining width
                    let mut see_all = config.see_all_enemies;
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.checkbox(&mut see_all, "See all enemies").changed() {
                            config.see_all_enemies = see_all;
                            commands.trigger(VisibilitiesUpdated);
                        }
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            seek_bar(ui, replay, &status);
                        });
                    });
                });
            });
        });

    Ok(())
}

fn seek_bar(ui: &mut egui::Ui, replay: &Handle, status: &Status) {
    let size = egui::vec2(ui.available_width(), BAR_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let total = status.total.as_secs_f32().max(f32::EPSILON);
    let progress = (status.position.as_secs_f32() / total).clamp(0., 1.);

    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    let radius = CornerRadius::same(3);
    painter.rect_filled(rect, radius, visuals.extreme_bg_color);
    let mut done = rect;
    done.set_width(rect.width() * progress);
    painter.rect_filled(done, radius, visuals.selection.bg_fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1., Color32::GRAY),
        StrokeKind::Inside,
    );

    if let Some(pointer) = response.hover_pos() {
        let ratio = ((pointer.x - rect.left()) / rect.width()).clamp(0., 1.);
        let at = status.total.mul_f32(ratio);
        response.clone().on_hover_text(time(at));
        if response.clicked() && !status.seeking {
            replay.seek(at);
        }
    }
}

fn speed(value: f32) -> String {
    format!("{value}x")
}

fn time(duration: Duration) -> String {
    let secs = duration.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}
