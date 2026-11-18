use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use oc_network::ToServer;
use oc_root::WcfgFrom;
use oc_root::geo::{ScreenVec2, WorldVec2};
use oc_utils::{let_ok, let_some};
use oc_world::support::SupportType;

use crate::cursor_to;
use crate::ingame::input::left_click::{LeftClick, LeftClickMode, SetLeftClick};
use crate::network::output::ToServerEvent;
use crate::states::GameConfig;

const RADIUS: f32 = 35.;

#[derive(Debug, Default, Resource)]
pub struct RequestedSupports(pub Vec<(SupportType, WorldVec2)>);

impl RequestedSupports {
    pub fn set(&mut self, type_: SupportType, position: WorldVec2) {
        self.0.retain(|(t, _)| t != &type_);
        self.0.push((type_, position));
    }

    pub fn remove(&mut self, type_: &SupportType) {
        self.0.retain(|(t, _)| t != type_);
    }
}

pub fn draw_requested(mut gizmos: Gizmos, g: Res<GameConfig>, requested: Res<RequestedSupports>) {
    let_some!(g = &g.0, return);

    for (_, position) in &requested.0 {
        let position = ScreenVec2::from_(*position, &g.w);
        gizmos.circle_2d(Vec2::new(position.x, position.y), RADIUS, RED);
    }
}

pub fn system(
    mut commands: Commands,
    mut gizmos: Gizmos,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    mode: Res<LeftClick>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !matches!(mode.0, LeftClickMode::Support(_)) {
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        commands.trigger(SetLeftClick(LeftClickMode::Select));
        return;
    }

    let_some!(cursor = window.cursor_position(), return);
    let (camera, transform) = *camera;
    let position = camera.viewport_to_world_2d(transform, cursor);
    let_ok!(position = position, return);

    gizmos.circle_2d(position, RADIUS, RED);
}

pub fn on_click(
    mut click: On<PointerClick>,
    g: Res<GameConfig>,
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    left_click: Res<LeftClick>,
    mut requested: ResMut<RequestedSupports>,
) {
    let_some!(g = &g.0, return);
    let LeftClickMode::Support(type_) = &left_click.0 else {
        return;
    };

    match click.button {
        PointerButton::Primary => {
            let point = click.pointer.position;
            let point = cursor_to!(point, camera, &g.w, WorldVec2);
            tracing::trace!(name="ingame-input-left-click-support-request", type_=?type_, point=?point);
            requested.set(type_.clone(), point);
            let support = ToServer::RequestSupport(type_.clone(), point);
            commands.trigger(ToServerEvent(support));
            commands.trigger(SetLeftClick(LeftClickMode::Select));
        }
        PointerButton::Secondary => {
            commands.trigger(SetLeftClick(LeftClickMode::Select));
        }
        PointerButton::Middle => {}
    }

    click.propagate(false);
}
