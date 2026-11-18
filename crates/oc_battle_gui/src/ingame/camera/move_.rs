use bevy::prelude::*;
use oc_root::{
    Wcfg, WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
    y::Y,
};
use oc_utils::{let_ok, let_some};

use crate::{
    ingame::camera::{
        self, GoToPoint, map::window_point_to_world_map_point, region::UpdateRegions,
    },
    states::GameConfig,
};

/// Pixels the cursor must move with right button pressed to start moving the map.
/// Used to permit contextual menu easly in case of little move.
const RIGHT_DRAG_THRESHOLD: f32 = 2.;

#[derive(Debug, Event, Deref)]
pub struct CenterCameraOn(pub WorldVec2);

#[derive(Debug, Event)]
pub struct MovedBattleCamera;

#[derive(Debug, Event)]
pub struct UpdateVisibleBattleSquare(pub Vec2); // The bevy world map point correspnding to the center of the screen

pub fn move_battle(
    mut commands: Commands,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<super::State>,
    mut input: ResMut<crate::ingame::input::State>,
) {
    let cursor = window.cursor_position();
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let left = buttons.pressed(MouseButton::Left);
    let right = buttons.pressed(MouseButton::Right);

    // Right press becomes a map drag only after a real move: tiny involuntary moves stay a click
    if let (true, Some(first), Some(cursor)) = (right, input.first_right_press, cursor) {
        input.right_dragging |= first.distance(cursor) > RIGHT_DRAG_THRESHOLD;
    }

    if ((right && input.right_dragging) || (left && ctrl))
        && let (Some(cursor1), Some(cursor2)) = (&state.cursor, &cursor)
    {
        let diff = cursor1 - cursor2;
        if diff != Vec2::ZERO {
            camera.translation.x += diff.x;
            camera.translation.y -= diff.y;
            tracing::trace!(name = "ingame-camera-trigger-moved-battle-camera");
            commands.trigger(MovedBattleCamera)
        }
    }
}

pub fn on_center_camera_on(
    point: On<CenterCameraOn>,
    mut commands: Commands,
    mut camera: Single<&mut Transform, With<Camera2d>>,
    g: Res<GameConfig>,
) {
    let_some!(g = &g.0, return);
    camera.translation.x = point.x;
    camera.translation.y = point.y.to_gui_y(&g.w);
    commands.trigger(MovedBattleCamera)
}

pub fn on_moved_battle_camera(
    _: On<MovedBattleCamera>,
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (camera, transform) = *camera;
    let width = window.resolution.width();
    let height = window.resolution.height();
    let center = Vec2::new(width / 2., height / 2.);
    let center = camera.viewport_to_world_2d(transform, center);
    let_ok!(center = center, return);

    commands.trigger(UpdateVisibleBattleSquare(center));
    commands.trigger(UpdateRegions(center));
}

pub fn move_in_world_map(
    mut commands: Commands,
    w: Res<Wcfg>,
    window: Single<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<camera::State>,
) {
    let_some!(w = &w.0, return);

    if buttons.just_released(MouseButton::Left) {
        let_some!(cursor = window.cursor_position(), return);

        let point = window_point_to_world_map_point(w, cursor, window.size());
        let center = Vec3::new(
            point.x - window.width() / 2.,
            point.y - window.height() / 2.,
            0.,
        );

        tracing::debug!("change battle camera center for {center:?}");
        state.previously = Some(Vec3::new(point.x, point.y, 0.0));

        tracing::debug!("Request update region for {point:?}");
        commands.trigger(UpdateRegions(point));
        commands.trigger(UpdateVisibleBattleSquare(point));
    }
}

pub fn on_go_to_point(
    point: On<GoToPoint>,
    g: Res<GameConfig>,
    mut commands: Commands,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let_some!(g = &g.0, return);
    tracing::debug!("Moved on {point:?}");
    let point = ScreenVec2::from_(point.0, &g.w);
    camera.translation.x = point.x;
    camera.translation.y = point.y;
    commands.trigger(UpdateRegions(point.into()));
    commands.trigger(UpdateVisibleBattleSquare(point.into()));
}
