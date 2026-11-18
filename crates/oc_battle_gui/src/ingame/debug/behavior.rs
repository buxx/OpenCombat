use bevy::prelude::*;
use oc_physics::update::bevy::Position;
use oc_root::WcfgFrom;
use oc_root::geo::ScreenVec2;

use crate::entity::individual::Behavior;
use crate::states::GameConfig;

const HIDE_ENGAGE_DISTANCE_COLOR: Color = Color::srgba(1.0, 0.5, 0.0, 0.3);

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct HideEngageDistanceGizmos;

pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-debug-behavior-setup");
    let (gizmos, _) = config.config_mut::<HideEngageDistanceGizmos>();
    gizmos.line.width = 1.0;
}

pub fn show_hide_engage_distances(
    g: Res<GameConfig>,
    individuals: Query<(&Behavior, &Position)>,
    mut gizmos: Gizmos<HideEngageDistanceGizmos>,
) {
    let Some(g) = &g.0 else { return };

    for (behavior, position) in &individuals {
        if !matches!(behavior.0, oc_individual::behavior::Behavior::Hide(_)) {
            continue;
        }

        let position = ScreenVec2::from_(position.0, &g.w);
        let position = Vec2::new(position.x, position.y);
        let radius = g.w.individual_hide_engage_distance().pixels(&g.w);
        gizmos.circle_2d(position, radius, HIDE_ENGAGE_DISTANCE_COLOR);
    }
}
