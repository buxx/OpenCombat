use bevy::prelude::*;
use oc_physics::update::bevy::Position;
use oc_root::{WcfgFrom, geo::ScreenVec3, y::V};
use oc_utils::{bevy::EntityMapping, let_ok, let_some};

use crate::{
    entity::individual::IndividualIndex,
    ingame::{
        InGameState,
        camera::{self, squad::ShowFormationPositions},
        individual::Gesture,
    },
    states::{AppState, GameConfig},
    world,
};

const FORMATION_POSITON_COLOR: Color = Color::srgba(0.5, 1.0, 0.0, 0.2);
const FORMATION_POSITON_COLOR_DEBUG: Color = Color::srgba(0.5, 1.0, 0.0, 1.0);

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<FormationPositionsGizmos>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                draw_formations
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle)),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct FormationPositionsGizmos;

pub fn setup(mut config: ResMut<GizmoConfigStore>) {
    tracing::trace!(name = "ingame-squad-debug-setup-formations-gizmos");
    let (gizmos, _) = config.config_mut::<FormationPositionsGizmos>();
    gizmos.line.width = 2.0;
}

#[allow(clippy::too_many_arguments)]
pub fn draw_formations(
    g: Res<GameConfig>,
    show: Res<ShowFormationPositions>,
    mut gizmos: Gizmos<FormationPositionsGizmos>,
    state: Res<camera::State>,
    world: Res<world::World>,
    individuals: Res<EntityMapping<oc_individual::IndividualIndex>>,
    individual: Query<(&Position, &Gesture), With<IndividualIndex>>,
    debug: Res<ShowFormationPositions>,
) {
    if !show.0 {
        return;
    }
    let_some!(g = &g.0, return);
    let_some!(regions = &state.regions, return);
    let color = match debug.0 {
        true => FORMATION_POSITON_COLOR_DEBUG,
        false => FORMATION_POSITON_COLOR,
    };

    tracing::trace!(name = "ingame-squad-debug-formations-draw", regions=?regions);
    for region in regions {
        let_some!(squads = world.squads.get(&region.0), return);

        for (i, squad) in squads {
            let leader = squad.leader();
            let count = squad.members.len(); // TODO: active members (compute must be cached in Squad)
            let_some!(leader = individuals.get(&leader), continue);
            let_ok!((position, gesture) = individual.get(*leader), continue);
            let position = ScreenVec3::from_(position.0, &g.w);
            let angle = gesture.0.direction().angle(V::Gui);
            let positions = squad.formation.positions(
                &g.w,
                // TODO: abnormal, should be Gui. Understand why.
                V::Server,
                glam::Vec2::new(position.x, position.y),
                angle,
                count,
            );

            for position in positions {
                tracing::trace!(name = "ingame-squad-debug-formations-draw-point", i=?i, position=?position);
                gizmos.circle_2d(bevy::math::Vec2::new(position.x, position.y), 1., color);
            }
        }
    }
}
