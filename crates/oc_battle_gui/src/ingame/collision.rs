use std::time::{Duration, Instant};

use bevy::prelude::*;
use oc_root::{geo::WorldVec3, y::Y};
use oc_utils::let_some;

use crate::states::GameConfig;

#[derive(Debug, Event)]
pub struct CollisionEvent(pub WorldVec3, pub Duration);

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct CollisionGizmos;

pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-collision-setup");
    let (gizmos, _) = config.config_mut::<CollisionGizmos>();
    gizmos.line.width = if cfg!(feature = "debug") { 2.0 } else { 1.0 };
}

pub fn on_collision(event: On<CollisionEvent>, mut state: ResMut<crate::ingame::state::State>) {
    state.collisions.push((Instant::now() + event.1, event.0));
}

pub fn show_collisions(
    g: Res<GameConfig>,
    mut state: ResMut<crate::ingame::state::State>,
    mut gizmos: Gizmos<CollisionGizmos>,
) {
    let_some!(g = &g.0, return);
    const DISPLAY_DURATION: Duration = Duration::from_millis(500);
    const COLOR: Color = if cfg!(feature = "debug") {
        Color::linear_rgb(1.0, 1.0, 0.0)
    } else {
        Color::srgba(0.5, 0.5, 0.5, 0.6)
    };

    let mut collisions = Vec::with_capacity(state.collisions.len());
    while let Some((instant, position)) = state.collisions.pop() {
        // Collision is displayed once its lag is elapsed
        let Some(elapsed) = Instant::now().checked_duration_since(instant) else {
            collisions.push((instant, position));
            continue;
        };
        if elapsed <= DISPLAY_DURATION {
            let position_ = Vec2::new(position.x, position.y.to_gui_y(&g.w));
            gizmos.circle_2d(position_, 1.0, COLOR);
            collisions.push((instant, position))
        }
    }
    state.collisions = collisions;
}
