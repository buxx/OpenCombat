use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// WRNING: AI generated module

const GLOW_SIZE: u32 = 64;

/// Soft round white texture (opacity in red channel) shared by particle effects
#[derive(Resource)]
pub struct GlowTexture(pub Handle<Image>);

/// Entity despawned (with its particle effect) when timer is finished
#[derive(Component)]
pub struct Lifetime(pub Timer);

pub fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let half = GLOW_SIZE as f32 / 2.0;
    let mut data = Vec::with_capacity((GLOW_SIZE * GLOW_SIZE * 4) as usize);
    for y in 0..GLOW_SIZE {
        for x in 0..GLOW_SIZE {
            let (dx, dy) = (x as f32 + 0.5 - half, y as f32 + 0.5 - half);
            let distance = ((dx * dx + dy * dy).sqrt() / half).min(1.0);
            let opacity = (1.0 - distance).powi(2);
            let value = (opacity * 255.0) as u8;
            data.extend_from_slice(&[value, 255, 255, value]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: GLOW_SIZE,
            height: GLOW_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::linear();
    commands.insert_resource(GlowTexture(images.add(image)));
}

pub fn despawn_finished(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut effects {
        if lifetime.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
