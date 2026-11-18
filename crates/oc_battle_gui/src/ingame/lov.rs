use bevy::prelude::*;

use derive_more::Deref;
use oc_geo::tile::TileXy;
use oc_mod::DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS;
use oc_root::WcfgFrom;
use oc_root::geo::{ScreenVec2, WorldVec2, WorldVec3};
use oc_root::{Wcfg, WorldConfig, physics::Meters};
use oc_utils::{d2::Xy, let_ok, let_some};
use strum_macros::{Display, EnumIter};

use crate::cursor_to;
use crate::ingame::input::left_click::LeftClick;

use crate::states::GameConfig;

use crate::{ingame::InGameState, ingame::draw, world::World};

#[derive(Debug, Event, Deref)]
pub struct SpawnBeginningLov(pub SpawnLovProfile);

#[derive(Debug, Event, Deref)]
pub struct SpawnLov(pub Lov);

#[derive(Debug, Event)]
pub struct UpdateLovFor(pub Entity, pub WorldVec2);

#[derive(Debug, Event)]
pub struct DespawnLov;

#[derive(Debug, Clone)]
pub struct SpawnLovConfig {
    pub click: LovClickMode,
    pub start_plus_z: Meters,
    pub stop_plus_z: Meters,
}

impl Default for SpawnLovConfig {
    fn default() -> Self {
        Self {
            click: LovClickMode::DraggedClick,
            start_plus_z: DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS,
            stop_plus_z: Meters(0.),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpawnLovProfile {
    pub start: WorldVec2,
    pub start_plus_z: Meters,
    pub stop_plus_z: Meters,
    pub max: Option<Meters>,
}

#[derive(Debug, Clone, Component)]
pub struct Lov {
    pub start: WorldVec3,
    pub stop: WorldVec3,
    pub stop_plus_z: Meters,
    pub sections: Vec<(WorldVec2, WorldVec2, Color)>,
    pub max: Option<Meters>,
}

#[cfg(feature = "debug")]
#[derive(Debug, Clone, Copy, Default, Display, EnumIter, PartialEq, Eq)]
pub enum SpawnProjectileClickMode {
    TwoClicks,
    #[default]
    DraggedClick,
}

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, PartialEq, Eq)]
pub enum LovClickMode {
    TwoClicks,
    #[default]
    DraggedClick,
    // SelectedSquads,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct LovGizmos;

pub struct LovPlugin;

impl Plugin for LovPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<LovGizmos>()
            .add_systems(Startup, setup)
            .add_observer(on_spawn_beginning_lov)
            .add_observer(on_spawn_lov)
            .add_observer(on_update_lov_for)
            .add_observer(on_despawn_lov)
            .add_systems(
                Update,
                (update_lov, draw_lovs).run_if(in_state(InGameState::Battle)),
            );
    }
}

fn setup(mut config: ResMut<GizmoConfigStore>) {
    let (gizmos, _) = config.config_mut::<LovGizmos>();
    gizmos.line.width = 5.0;
}

fn on_spawn_beginning_lov(
    spawn: On<SpawnBeginningLov>,
    w: Res<Wcfg>,
    mut commands: Commands,
    world: Res<World>,
) {
    tracing::trace!(name = "lov-spawn-beginning", spawn=?spawn);
    let_some!(w = &w.0, return);
    let_some!(tile = world.tile_at(w, spawn.start), return);
    let z = tile.z_pixels(w) + spawn.start_plus_z.0 * w.geo_pixels_per_meters();
    let start = spawn.start.extend(z);

    tracing::trace!(name = "lov-spawn-beginning", start=?start);
    commands.spawn(Lov {
        start,
        stop: start,
        stop_plus_z: spawn.stop_plus_z,
        sections: vec![],
        max: spawn.max,
    });
}

fn on_spawn_lov(spawn: On<SpawnLov>, mut commands: Commands) {
    tracing::trace!(name = "lov-spawn", lov=?spawn.0);
    commands.spawn(spawn.0.clone());
}

fn update_lov(
    g: Res<GameConfig>,
    mut commands: Commands,
    lovs: Query<Entity, With<Lov>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mode: Res<LeftClick>,
) {
    let_some!(g = &g.0, return);
    if !mode.0.display_lov() {
        return;
    }
    let_some!(cursor = window.cursor_position(), return);
    let position = cursor_to!(cursor, camera, &g.w, WorldVec2);

    for lov in lovs {
        tracing::trace!(name="update-lov-trigger-for", lov=?lov, position=?position);
        commands.trigger(UpdateLovFor(lov, position));
    }
}

fn draw_lovs(g: Res<GameConfig>, lovs: Query<&Lov>, mut gizmos: Gizmos) {
    let_some!(g = &g.0, return);

    for lov in lovs {
        for (start, end, color) in &lov.sections {
            let start = ScreenVec2::from_(*start, &g.w);
            let end = ScreenVec2::from_(*end, &g.w);
            gizmos.line(
                start.extend(draw::Z_LOV).into(),
                end.extend(draw::Z_LOV).into(),
                *color,
            );
        }
    }
}

fn on_update_lov_for(
    update: On<UpdateLovFor>,
    g: Res<GameConfig>,
    mut lovs: Query<&mut Lov>,
    world: Res<World>,
) {
    tracing::trace!(name = "on-update-lov-for-try");
    let_some!(g = &g.0, return);
    let (lov, position) = (update.0, update.1);
    let_ok!(mut lov = lovs.get_mut(lov), return);

    let start = lov.start;
    let stop_tile = world.tile_at(&g.w, position);
    let_some!(stop_tile = stop_tile, return);
    tracing::trace!(name = "on-update-lov-for");

    let stop = position.extend(stop_tile.z_pixels(&g.w) + lov.stop_plus_z.pixels(&g.w));
    let at = |xy, z| path_objects_at(&g.w, &g.mod_, &world, xy, z);
    let modifier = oc_lov::Modifier::None(false);
    let path = oc_lov::PathBuilder::new(&g.w, at).build(start, stop, 0, modifier);

    let max_distance = lov.max.map(|m| m.pixels(&g.w));
    let origin = Vec2::from(WorldVec2::from(start));
    let sections = path
        .sections
        .iter()
        .flat_map(|section| {
            let color = Color::srgb(0.0 + section.opacity.0, 1.0 - section.opacity.0, 0.0);
            let (start, stop) = (section.start.into(), section.stop.into());
            tracing::trace!(name = "on-update-lov-for-section", start=?start, stop=?stop, color=?color);
            clip_section(start, stop, color, origin, max_distance)
        })
        .collect();

    lov.start = start;
    lov.stop = stop;
    lov.sections = sections;
}

/// Part of the section further than max distance from origin is black
fn clip_section(
    start: WorldVec2,
    stop: WorldVec2,
    color: Color,
    origin: Vec2,
    max_distance: Option<f32>,
) -> Vec<(WorldVec2, WorldVec2, Color)> {
    let Some(max) = max_distance else {
        return vec![(start, stop, color)];
    };

    let (a, b) = (Vec2::from(start), Vec2::from(stop));
    let (from, to) = (a.distance(origin), b.distance(origin));

    if to <= max {
        vec![(start, stop, color)]
    } else if from >= max {
        vec![(start, stop, Color::BLACK)]
    } else {
        let cut = a.lerp(b, (max - from) / (to - from));
        let cut = WorldVec2::new(cut.x, cut.y);
        vec![(start, cut, color), (cut, stop, Color::BLACK)]
    }
}

fn path_objects_at(
    w: &WorldConfig,
    mod_: &oc_mod::Mod,
    world: &World,
    at: Xy,
    z: f32,
) -> Vec<oc_lov::Step> {
    world
        .tile(w, TileXy(at))
        .map(|t| {
            let tile_z = t.z as f32 * w.geo_meters_per_z().0 * w.geo_pixels_per_meters();
            let relative_z = z - tile_z;
            let nature = mod_.nature(t.nature);
            let mut steps = vec![oc_lov::Step::from_nature(w, nature, relative_z)];
            let smoke = world.smoke_at(TileXy(at), w);
            if !smoke.is_zero() {
                steps.push(oc_lov::Step::from_smoke(smoke));
            }
            steps
        })
        .unwrap_or(vec![])
}

fn on_despawn_lov(_: On<DespawnLov>, mut commands: Commands, lovs: Query<(Entity, &Lov)>) {
    tracing::trace!(name = "lov-despawn");
    for (entity, _lov) in lovs {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(start: f32, stop: f32, max: Option<f32>) -> Vec<(f32, f32, Color)> {
        let (start, stop) = (WorldVec2::new(start, 0.), WorldVec2::new(stop, 0.));
        clip_section(start, stop, Color::WHITE, Vec2::ZERO, max)
            .into_iter()
            .map(|(a, b, color)| (a.x, b.x, color))
            .collect()
    }

    #[test]
    fn clip_without_max_keeps_section() {
        assert_eq!(clip(10., 50., None), vec![(10., 50., Color::WHITE)]);
    }

    #[test]
    fn clip_section_inside_max_keeps_color() {
        assert_eq!(clip(10., 50., Some(50.)), vec![(10., 50., Color::WHITE)]);
    }

    #[test]
    fn clip_section_beyond_max_is_black() {
        assert_eq!(clip(60., 80., Some(50.)), vec![(60., 80., Color::BLACK)]);
    }

    #[test]
    fn clip_section_across_max_is_split() {
        assert_eq!(
            clip(10., 90., Some(50.)),
            vec![(10., 50., Color::WHITE), (50., 90., Color::BLACK)]
        );
    }
}
