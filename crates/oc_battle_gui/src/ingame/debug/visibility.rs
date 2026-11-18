use bevy::prelude::*;
use oc_root::physics::Meters;
use oc_utils::let_some;

use crate::ingame::lov::{DespawnLov, Lov, SpawnLov};

#[derive(Debug, Event)]
pub struct ToggleShowVisibility;

#[derive(Debug, Resource, Deref, DerefMut, Default)]
pub struct ShowVisibility(pub bool);

pub fn on_toggle_show_formation_positions(
    _: On<ToggleShowVisibility>,
    mut commands: Commands,
    mut show: ResMut<ShowVisibility>,
) {
    show.0 = !show.0;
    if !show.0 {
        commands.trigger(DespawnLov);
    }
}

pub fn show_visibilities(
    mut commands: Commands,
    show: Res<ShowVisibility>,
    world: Res<crate::world::World>,
) {
    if !show.0 {
        return;
    }

    // Clear all before re-create all
    commands.trigger(DespawnLov);

    for (i1, i2, visibility) in &world.visibilities.all {
        let_some!(individual1 = world.individual(*i1), continue);
        let_some!(individual2 = world.individual(*i2), continue);
        let start = individual1.position;
        let stop = individual2.position;
        let sections = visibility
            .sections
            .clone()
            .into_iter()
            .map(|(start, stop, opacity)| {
                (
                    start.into(),
                    stop.into(),
                    Color::srgb(0.0 + opacity.0, 1.0 - opacity.0, 0.0),
                )
            })
            .collect::<Vec<_>>();
        commands.trigger(SpawnLov(Lov {
            start,
            stop,
            stop_plus_z: Meters(0.), // Field was used for beginning lov
            sections,
            max: None,
        }));
    }
}
