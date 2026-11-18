use bevy::prelude::*;
use oc_network::GameConfig;
use oc_root::{
    WcfgFrom,
    files::Files,
    geo::{ScreenVec2, WorldVec2},
    physics::Meters,
};
use oc_utils::let_some;

use crate::ingame::draw::Z_EXPLOSION;

const FRAME_SIZE: UVec2 = UVec2::new(196, 190);
const FRAMES: u32 = 11;
const FRAME_DURATION: f32 = 0.15;
pub const ANIMATION_DURATION: f32 = FRAMES as f32 * FRAME_DURATION;

#[derive(Component)]
pub struct ExplosionAnimation(Timer);

pub fn spawn(
    commands: &mut Commands,
    assets: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    g: &GameConfig,
    files: &Files,
    position: WorldVec2,
    rayon: Meters,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        FRAME_SIZE, FRAMES, 1, None, None,
    ));
    let sprite_path = files.sprites().join("explosions.png");
    tracing::debug!(name = "fx-explosion-spawn", path=?sprite_path, position=?position);
    let image = assets.load(sprite_path);
    let position = ScreenVec2::from_(position, &g.w);
    let width = rayon.pixels(&g.w) * 2.;
    let height = width * FRAME_SIZE.y as f32 / FRAME_SIZE.x as f32;

    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(width, height)),
            ..Sprite::from_atlas_image(image, TextureAtlas { layout, index: 0 })
        },
        Transform::from_xyz(position.x, position.y, Z_EXPLOSION),
        ExplosionAnimation(Timer::from_seconds(FRAME_DURATION, TimerMode::Repeating)),
    ));
}

pub fn animate(
    mut commands: Commands,
    time: Res<Time>,
    mut explosions: Query<(Entity, &mut ExplosionAnimation, &mut Sprite)>,
) {
    for (entity, mut animation, mut sprite) in &mut explosions {
        if !animation.0.tick(time.delta()).just_finished() {
            continue;
        }
        let_some!(atlas = &mut sprite.texture_atlas, continue);
        if atlas.index + 1 >= FRAMES as usize {
            commands.entity(entity).despawn();
        } else {
            atlas.index += 1;
        }
    }
}

#[cfg(feature = "debug")]
#[derive(Component)]
pub struct ExplosionDebugDisplay {
    timer: Timer,
    center: Vec2,
    kill_rayon: f32,
    stunned_rayon: f32,
    shrapnel_rayon: f32,
    destruction_rayon: f32,
    shrapnels: Vec<(Vec2, Vec2)>,
}

#[cfg(feature = "debug")]
pub fn spawn_debug(
    commands: &mut Commands,
    g: &GameConfig,
    debug: &oc_physics::fx::ExplosionDebug,
) {
    commands.spawn(ExplosionDebugDisplay {
        timer: Timer::from_seconds(ANIMATION_DURATION, TimerMode::Once),
        center: ScreenVec2::from_(debug.position, &g.w).into(),
        kill_rayon: debug.kill_rayon.pixels(&g.w),
        stunned_rayon: debug.stunned_rayon.pixels(&g.w),
        shrapnel_rayon: debug.shrapnel_rayon.pixels(&g.w),
        destruction_rayon: debug.destruction_rayon.pixels(&g.w),
        shrapnels: debug
            .shrapnels
            .iter()
            .map(|(a, b)| {
                (
                    ScreenVec2::from_(*a, &g.w).into(),
                    ScreenVec2::from_(*b, &g.w).into(),
                )
            })
            .collect(),
    });
}

#[cfg(feature = "debug")]
pub fn draw_debug(
    mut commands: Commands,
    time: Res<Time>,
    mut displays: Query<(Entity, &mut ExplosionDebugDisplay)>,
    mut gizmos: Gizmos,
) {
    for (entity, mut display) in &mut displays {
        if display.timer.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let center = display.center;
        gizmos.circle_2d(center, display.kill_rayon, Color::srgb(1., 0., 0.));
        gizmos.circle_2d(center, display.stunned_rayon, Color::srgb(1., 0.6, 0.));
        gizmos.circle_2d(center, display.shrapnel_rayon, Color::srgb(1., 1., 0.));
        gizmos.circle_2d(center, display.destruction_rayon, Color::srgb(0.7, 0.3, 1.));

        for (a, b) in &display.shrapnels {
            gizmos.line_2d(*a, *b, Color::srgb(1., 1., 1.));
        }
    }
}
