use bevy::prelude::*;
use bevy_hanabi::Gradient;
use bevy_hanabi::prelude::*;
use oc_network::GameConfig;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
    physics::Meters,
};

use super::explosion;
use super::particles::{GlowTexture, Lifetime};
use crate::ingame::draw::Z_EXPLOSION;

// WRNING: AI generated module

/// Flash is drawn above the explosion sprite
const Z_FLASH: f32 = Z_EXPLOSION + 0.1;
/// Flash covers this factor of the largest explosion rayon
const RAYON_FACTOR: f32 = 0.5;
/// Factor applied to animation duration for flash
const DURACTION_FACTOR: f32 = 0.33;

/// Light flash at explosion center: an additive soft glow, as large as the largest explosion
/// rayon, lasting as long as the explosion animation.
pub fn spawn(
    commands: &mut Commands,
    effects: &mut Assets<EffectAsset>,
    glow: &GlowTexture,
    g: &GameConfig,
    position: WorldVec2,
    rayon: Meters,
) {
    let duration = explosion::ANIMATION_DURATION * DURACTION_FACTOR;
    let diameter = rayon.pixels(&g.w) * 2. * RAYON_FACTOR;
    let position = ScreenVec2::from_(position, &g.w);

    let mut color = Gradient::new();
    color.add_key(0.0, Vec4::new(1.0, 0.9, 0.6, 1.0));
    color.add_key(0.15, Vec4::new(1.0, 0.75, 0.4, 0.6));
    color.add_key(1.0, Vec4::new(1.0, 0.5, 0.2, 0.0));

    let mut size = Gradient::new();
    size.add_key(0.0, Vec3::splat(diameter * 0.6));
    size.add_key(0.15, Vec3::splat(diameter));
    size.add_key(1.0, Vec3::splat(diameter * 0.8));

    let writer = ExprWriter::new();
    let init_pos = SetAttributeModifier::new(Attribute::POSITION, writer.lit(Vec3::ZERO).expr());
    let init_age = SetAttributeModifier::new(Attribute::AGE, writer.lit(0.).expr());
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, writer.lit(duration).expr());
    let mut module = writer.finish();
    module.add_texture_slot("glow", SlotDimension::D2);

    let effect = EffectAsset::new(1, SpawnerSettings::once(1.0.into()), module)
        .with_name("flash")
        .with_alpha_mode(bevy_hanabi::AlphaMode::Add)
        .with_motion_integration(MotionIntegration::None)
        .init(init_pos)
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
        Transform::from_xyz(position.x, position.y, Z_FLASH),
        Lifetime(Timer::from_seconds(duration + 0.1, TimerMode::Once)),
    ));
}
