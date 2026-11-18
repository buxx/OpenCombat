use crate::states::GuiFiles;
use crate::{
    ingame::hud::{GAP, HUD_HEIGHT},
    network,
    states::GameConfig,
};

use bevy::prelude::*;
use oc_individual::Status;
use oc_root::geo::WorldVec2;
use oc_utils::{every, let_some};
use std::time::Duration;

pub const MINIMAP_SIZE: f32 = HUD_HEIGHT - 2. * GAP;
const MARKER_SIZE: f32 = 4.;
const MARKER_REFRESH: Duration = Duration::from_secs(3);
const VIEWPORT_COLOR: Color = Color::srgba(0.9, 0.9, 0.9, 0.7);
const SQUAD_MARKER_COLOR_OPPOSITE: Color = Color::srgb(1., 0.1, 0.1);
const SQUAD_MARKER_COLOR_OWNED: Color = Color::srgb(0.2, 0.4, 1.);

#[derive(Component, Default, Clone)]
pub struct Minimap;

#[derive(Component, Default, Clone)]
pub struct MinimapMarker;

#[derive(Component, Default, Clone)]
pub struct MinimapViewport;

pub fn minimap() -> impl Scene {
    bsn! {
        Minimap
        Node {
            width: px(MINIMAP_SIZE),
            height: px(MINIMAP_SIZE),
            flex_shrink: 0.,
        }
        BackgroundColor(Color::BLACK)
        Children [
            MinimapViewport
            Node {
                position_type: PositionType::Absolute,
                border: UiRect::all(px(1)),
            }
            BorderColor::all(VIEWPORT_COLOR)
            Pickable::IGNORE
        ]
    }
}

pub fn load_image(
    mut commands: Commands,
    slot: Single<Entity, (With<Minimap>, Without<ImageNode>)>,
    assets: Res<AssetServer>,
    files: Res<GuiFiles>,
) {
    let_some!(files = &files.0, return);
    commands.entity(*slot).insert(ImageNode::new(
        assets.load(files.asset(oc_root::files::File::HudMinimap)),
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn update_squad_markers(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    minimap: Single<Entity, With<Minimap>>,
    markers: Query<Entity, With<MinimapMarker>>,
    world: Res<crate::world::World>,
    g: Res<GameConfig>,
    network: Res<network::state::State>,
    config: Res<crate::config::Config>,
) {
    every!(timer, time, MARKER_REFRESH);
    let_some!(g = &g.0, return);
    let_some!(identity = &network.identity, return);
    let world_width = g.w.world_width_pixels() as f32;
    let world_height = g.w.world_height_pixels() as f32;
    let mut points: Vec<(WorldVec2, Color)> = vec![];

    // TODO: bevy 0.20: use https://docs.rs/bevy/0.20.0/bevy/ecs/system/command/fn.despawn_all.html
    for marker in &markers {
        commands.entity(marker).despawn();
    }

    let all = config.see_all_enemies;
    points.extend(owned_squad_points(&world, identity.side));
    points.extend(opposite_squad_points(world, identity.side.opposite(), all));

    for (position, color) in points {
        let x = (position.x / world_width).clamp(0., 1.) * MINIMAP_SIZE;
        let y = (position.y / world_height).clamp(0., 1.) * MINIMAP_SIZE;
        commands
            .spawn_scene(marker(x, y, color))
            .insert(ChildOf(*minimap));
    }
}

fn marker(x: f32, y: f32, color: Color) -> impl Scene {
    bsn! {
        MinimapMarker
        Node {
            position_type: PositionType::Absolute,
            left: {px(x - MARKER_SIZE / 2.)},
            top: {px(y - MARKER_SIZE / 2.)},
            width: px(MARKER_SIZE),
            height: px(MARKER_SIZE),
        }
        BackgroundColor({color})
        Pickable::IGNORE
    }
}

fn opposite_squad_points(
    world: Res<'_, crate::world::World>,
    side: oc_root::side::Side,
    see_all: bool,
) -> Vec<(WorldVec2, Color)> {
    let mut points = vec![];

    for (_, squad) in world.side_squads(side) {
        let visible = squad
            .members
            .iter()
            .any(|i| world.operational(*i) && (see_all || world.visible(*i)));
        if visible {
            let_some!(leader = world.individual(squad.leader()), continue);
            points.push((leader.position.into(), SQUAD_MARKER_COLOR_OPPOSITE));
        }
    }

    points
}

fn owned_squad_points(
    world: &Res<'_, crate::world::World>,
    side: oc_root::side::Side,
) -> Vec<(WorldVec2, Color)> {
    let mut points = vec![];

    for (_, squad) in world.side_squads(side) {
        let visible = squad.members.iter().any(|i| {
            world
                .individual(*i)
                .is_some_and(|individual| individual.status == Status::Operational)
        });
        if visible {
            let_some!(leader = world.individual(squad.leader()), continue);
            points.push((leader.position.into(), SQUAD_MARKER_COLOR_OWNED));
        }
    }

    points
}

pub fn update_viewport(
    mut viewport: Single<&mut Node, With<MinimapViewport>>,
    transform: Single<&Transform, With<Camera2d>>,
    window: Single<&Window>,
    g: Res<GameConfig>,
) {
    let_some!(g = &g.0, return);
    let world = Vec2::new(
        g.w.world_width_pixels() as f32,
        g.w.world_height_pixels() as f32,
    );
    let size = window.size();
    let center = Vec2::new(transform.translation.x, world.y - transform.translation.y);
    let top_left = (center - size / 2.) / world * MINIMAP_SIZE;
    let size = (size / world * MINIMAP_SIZE).min(Vec2::splat(MINIMAP_SIZE));
    let left = top_left.x.max(0.);
    let top = top_left.y.max(0.);
    let width = size.x.min(MINIMAP_SIZE - left);
    let height = size.y.min(MINIMAP_SIZE - top);

    viewport.left = px(left);
    viewport.top = px(top);
    viewport.width = px(width);
    viewport.height = px(height);
}
