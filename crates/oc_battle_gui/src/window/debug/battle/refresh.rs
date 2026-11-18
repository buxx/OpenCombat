use std::time::Instant;

use bevy::prelude::*;
use oc_geo::{region::RegionXy, tile::TileXy};
use oc_physics::update::bevy::{Forces, Position, Region, Tile};
use oc_root::y::Y;
use oc_root::{Wcfg, WcfgInto};
use oc_utils::let_some;

use crate::ingame;
use crate::ingame::individual::{Status, Suppress};
use crate::window::debug::battle::component::ComponentDetails;
use crate::world::World;
use crate::{
    entity::{individual::IndividualIndex, projectile::ProjectileId},
    ingame::camera::State,
    states,
    window::debug::{physics::PhysicsRepr, subject::Subject},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Refresh {
    #[default]
    EachFrame,
    X100ms,
    X1s,
}

impl Refresh {
    fn as_millis(&self) -> u128 {
        match self {
            Refresh::EachFrame => 0,
            Refresh::X100ms => 100,
            Refresh::X1s => 1000,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn on_refresh(
    _: On<super::Refresh>,
    w: Res<Wcfg>,
    g: Res<crate::states::GameConfig>,
    mut window: ResMut<states::Window>,
    camera: Res<State>,
    individuals: Query<(
        &IndividualIndex,
        &Position,
        &Tile,
        &Region,
        &Forces,
        &Status,
        &Suppress,
    )>,
    projectiles: Query<(&ProjectileId, &Position, &Tile, &Region, &Forces)>,
    window_: Single<&bevy::window::Window>,
    camera_: Single<(&Camera, &GlobalTransform)>,
    world: Res<World>,
    ingame: Res<ingame::state::State>,
) {
    let_some!(w = &w.0, return);
    let_some!(g = &g.0, return);

    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let Some(crate::window::Window::BattleDebug(window)) = &mut window.0 {
        window.context.regions = camera.regions.clone().unwrap_or(vec![]);
        window.context.individuals = individuals
            .iter()
            .map(|(id, position, _, region, _, status, suppress)| {
                Subject::new(
                    id.0,
                    PhysicsRepr::new(
                        position.0,
                        // tile.0.clone(),
                        region.0.into_(w),
                        // forces.0.clone(),
                    ),
                    vec![
                        ("Status".to_string(), format!("{:?}", status.0)),
                        ("Suppress".to_string(), format!("{:?}", suppress.0)),
                        (
                            "Tile".to_string(),
                            world
                                .tile_at(w, position.0.into())
                                .map(|t| g.mod_.nature(t.nature).name.clone())
                                .unwrap_or("?".to_string()),
                        ),
                    ],
                    ComponentDetails::Individual(id.0),
                )
            })
            .collect();
        window.context.projectiles = projectiles
            .iter()
            .map(|(id, position, _, region, _)| {
                Subject::new(
                    id.0,
                    PhysicsRepr::new(
                        position.0,
                        // tile.0.clone(),
                        region.0.into_(w),
                        // forces.0.clone(),
                    ),
                    vec![],
                    ComponentDetails::Projectile(id.0),
                )
            })
            .collect();

        let (camera, transform) = *camera_;
        if let Some(cursor) = window_.cursor_position()
            && let Ok(bevyp) = camera.viewport_to_world_2d(transform, cursor)
        {
            let point = Vec2::new(bevyp.x, bevyp.y.to_world_y(w));
            let tile: TileXy = [point.x, point.y].into_(w);
            let tile_ = world.tile(w, tile);
            let tile_ = tile_.map(|t| format!("{t:?}")).unwrap_or_default();
            let region: RegionXy = tile.into_(w);

            window.context.cursor = Some(cursor);
            window.context.point = Some(point);
            window.context.tile = Some((tile, tile_));
            window.context.region = Some(region);
        }

        window.context.ingame = ingame.clone();

        window.last = Instant::now();
    }
}

pub fn trigger_refresh(window: Res<states::Window>, mut commands: Commands) {
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let Some(crate::window::Window::BattleDebug(window)) = &window.0
        && window.last.elapsed().as_millis() > window.context.refresh.as_millis()
    {
        commands.trigger(super::Refresh);
    }
}
