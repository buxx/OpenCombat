use bevy::prelude::*;
use oc_network::ToServer;
use oc_root::geo::WorldVec2;
use oc_root::{Wcfg, WcfgFrom};
use oc_utils::let_some;

use crate::cursor_to;
use crate::ingame::input::left_click::{LeftClick, LeftClickMode, SetLeftClick};
use crate::network::output::ToServerEvent;

pub fn system(
    mut commands: Commands,
    w: Res<Wcfg>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mode: Res<LeftClick>,
) {
    let_some!(w = &w.0, return);
    let LeftClickMode::Explosion(shell) = &mode.0 else {
        return;
    };

    if keys.just_pressed(KeyCode::Escape) || buttons.just_pressed(MouseButton::Middle) {
        commands.trigger(SetLeftClick(LeftClickMode::Select));
        return;
    }

    if buttons.just_released(MouseButton::Left) {
        let_some!(cursor = window.cursor_position(), return);
        let point = cursor_to!(cursor, camera, w, WorldVec2);
        tracing::debug!("Debug explosion of shell {shell:?} at {point:?}");
        commands.trigger(ToServerEvent(ToServer::DebugExplosion(*shell, point)));
    }
}
