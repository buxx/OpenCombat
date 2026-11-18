use bevy::prelude::*;
use oc_network::ToServer;
use oc_root::geo::WorldVec2;
use oc_root::physics::Meters;
use oc_root::{Wcfg, WcfgFrom, WorldConfig};
use oc_utils::{let_some, return_if};

use crate::ingame::debug::projectile::SpawnProjectileProfile;
use crate::ingame::input::left_click::{
    DespawnClicksLine, LeftClick, LeftClickMode, SetLeftClick, SpawnClicksLine,
    SpawnProjectileLeftClick,
};
use crate::ingame::lov::SpawnProjectileClickMode;
use crate::network::output::ToServerEvent;
use crate::projectile::IntoSpawnProjectile;
use crate::world::World;
use crate::{cursor_to, ingame};

#[allow(clippy::too_many_arguments)]
pub fn system(
    mut commands: Commands,
    w: Res<Wcfg>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    mode: Res<LeftClick>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<crate::ingame::input::State>,
    spawn: Res<SpawnProjectileLeftClick>,
    world: Res<crate::world::World>,
) {
    let_some!(w = &w.0, return);
    let_some!(cursor = window.cursor_position(), return);
    let point = cursor_to!(cursor, camera, w, WorldVec2);
    let LeftClickMode::SpawnProjectile(profile) = &mode.0 else {
        return;
    };

    return_if!(maybe_cancel(&mut commands, &buttons, &keys));
    show(
        w,
        point,
        &mut commands,
        &buttons,
        &spawn,
        &mut state,
        &world,
        profile,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn show(
    w: &WorldConfig,
    point: WorldVec2,
    commands: &mut Commands,
    buttons: &ButtonInput<MouseButton>,
    spawn_projectile_mode: &SpawnProjectileLeftClick,
    state: &mut ingame::input::State,
    world: &World,
    profile: &SpawnProjectileProfile,
) {
    match spawn_projectile_mode.0 {
        SpawnProjectileClickMode::TwoClicks => {
            if buttons.just_released(MouseButton::Left) {
                state.clicks.push(point);

                if state.clicks.len() == 1 {
                    commands.trigger(SpawnClicksLine);
                }

                if state.clicks.len() == 2 {
                    let start = state.clicks.first().expect("len checked line before");
                    let end = state.clicks.last().expect("len checked line before");

                    if let (Some(start), Some(end)) = (
                        world.d2_to_d3(w, *start, profile.plus_z),
                        world.d2_to_d3(w, *end, Meters(0.)),
                    ) {
                        let spawn = profile.spawn(start, end);
                        tracing::debug!("Spawn projectile {spawn:?}");
                        commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
                    }

                    state.clicks.clear();
                }
            }
        }
        SpawnProjectileClickMode::DraggedClick => {
            if buttons.just_pressed(MouseButton::Left) {
                state.clicks.push(point);
                commands.trigger(SpawnClicksLine);
            }

            if buttons.just_released(MouseButton::Left) {
                if let Some(start) = state.clicks.first()
                    && let (Some(start), Some(end)) = (
                        world.d2_to_d3(w, *start, profile.plus_z),
                        world.d2_to_d3(w, point, Meters(0.)),
                    )
                {
                    let spawn = profile.spawn(start, end);
                    tracing::debug!("Spawn projectile {spawn:?}");
                    commands.trigger(ToServerEvent(ToServer::ExplodeProjectile(spawn)));
                }

                commands.trigger(DespawnClicksLine);
                state.clicks.clear();
            }
        }
    }
}

fn maybe_cancel(
    commands: &mut Commands,
    buttons: &ButtonInput<MouseButton>,
    keys: &ButtonInput<KeyCode>,
) -> bool {
    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Middle) {
        // TODO: need despawn things about line display
        commands.trigger(SetLeftClick(LeftClickMode::Select));
        return true;
    }
    false
}
