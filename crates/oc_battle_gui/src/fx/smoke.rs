use bevy::prelude::*;
use bevy_hanabi::Gradient;
use bevy_hanabi::prelude::*;
use oc_network::GameConfig;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
    physics::Meters,
};

use super::particles::{GlowTexture, Lifetime};
use crate::ingame::draw::Z_EXPLOSION;

// WRNING: AI generated module

/// Smoke cloud entity
#[derive(Component)]
pub struct Smoke;

/// Smoke is drawn above individuals and explosions
const Z_SMOKE: f32 = Z_EXPLOSION + 0.2;
/// Smoke cloud diameter
const SIZE: Meters = Meters(20.0);
const PUFFS: f32 = 48.0;
/// Puffs are born in a disc of this fraction of cloud rayon, then drift and grow to reach the
/// cloud edge
const SPAWN_RAYON_FACTOR: f32 = 0.3;
/// Maximum drift speed of puffs (random between 0 and this value)
const DRIFT_SPEED: Meters = Meters(0.08);
const MAX_ALPHA: f32 = 0.4;

/// Minimum fade time: avoid a gradient with two keys at the same place
const MIN_FADE: f32 = 0.1;

/// Smoke grenade cloud: puffs of soft grey particles which grow, keep their opacity during `hold`
/// seconds, then fade during `fade` seconds. Its limit never goes over [`SIZE`].
pub fn spawn(
    commands: &mut Commands,
    effects: &mut Assets<EffectAsset>,
    glow: &GlowTexture,
    g: &GameConfig,
    position: WorldVec2,
    hold: f32,
    fade: f32,
) {
    let fade = fade.max(MIN_FADE);
    let duration = hold.max(0.) + fade;
    let faded_from = hold.max(0.) / duration;
    let rayon = SIZE.pixels(&g.w) / 2.;
    let position = ScreenVec2::from_(position, &g.w);
    let drift_speed = DRIFT_SPEED.pixels(&g.w);
    // Puff rayon at the end is what remains between spawn disc, drift and cloud edge
    let end_diameter = (rayon * (1.0 - SPAWN_RAYON_FACTOR) - drift_speed * duration) * 2.;

    let mut color = Gradient::new();
    color.add_key(0.0, Vec4::new(0.8, 0.8, 0.8, 0.0));
    color.add_key(0.05, Vec4::new(0.8, 0.8, 0.8, MAX_ALPHA));
    color.add_key(
        faded_from.max(0.06),
        Vec4::new(0.7, 0.7, 0.7, MAX_ALPHA * 0.8),
    );
    color.add_key(1.0, Vec4::new(0.6, 0.6, 0.6, 0.0));

    let mut size = Gradient::new();
    size.add_key(0.0, Vec3::splat(end_diameter * 0.5));
    size.add_key(1.0, Vec3::splat(end_diameter));

    let writer = ExprWriter::new();
    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(rayon * SPAWN_RAYON_FACTOR).expr(),
        dimension: ShapeDimension::Volume,
    };
    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(drift_speed)).expr(),
    };
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    // Puffs fade out exactly at the end of cloud duration
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(duration).expr());
    let mut module = writer.finish();
    module.add_texture_slot("glow", SlotDimension::D2);

    let effect = EffectAsset::new(PUFFS as u32, SpawnerSettings::once(PUFFS.into()), module)
        .with_name("smoke")
        .with_alpha_mode(bevy_hanabi::AlphaMode::Blend)
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .render(ParticleTextureModifier {
            texture_slot: 0,
            sample_mapping: ImageSampleMapping::ModulateOpacityFromR,
        })
        .render(ColorOverLifetimeModifier::new(color))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        });

    commands.spawn((
        ParticleEffect::new(effects.add(effect)),
        EffectMaterial {
            images: vec![glow.0.clone()],
        },
        Transform::from_xyz(position.x, position.y, Z_SMOKE),
        Smoke,
        Lifetime(Timer::from_seconds(duration + 0.1, TimerMode::Once)),
    ));
}
