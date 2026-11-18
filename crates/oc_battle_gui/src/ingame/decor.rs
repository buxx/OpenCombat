use bevy::{prelude::*, sprite::Anchor};
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_physics::update::bevy::Region;
use oc_root::{WcfgInto, y::Y};
use oc_utils::{d2::Xy, let_some};
use oc_world::decor::Decor;

use crate::{
    ingame::{draw::Z_DECOR, region::ForgottenRegion, tile::rotation},
    sprites::decor::DecorSprites,
    states::GameConfig,
    world::{InsertDecor, InsertedTiles, RemoveDecor},
};

const Z_DECOR_SPAN: f32 = 0.05;

pub struct DecorPlugin;

impl Plugin for DecorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DecorsVisible>()
            .add_systems(Update, toggle_decors)
            .add_observer(on_inserted_tiles)
            .add_observer(on_insert_decor)
            .add_observer(on_remove_decor)
            .add_observer(on_forgotten_region);
    }
}

#[derive(Debug, Resource)]
struct DecorsVisible(bool);

impl Default for DecorsVisible {
    fn default() -> Self {
        Self(true)
    }
}

/// Press T to show/hide decors.
fn toggle_decors(
    keys: Res<ButtonInput<KeyCode>>,
    mut visible: ResMut<DecorsVisible>,
    mut sprites: Query<&mut Visibility, With<DecorSprite>>,
) {
    if keys.just_pressed(KeyCode::KeyT) {
        visible.0 = !visible.0;
        for mut v in &mut sprites {
            *v = visibility(visible.0);
        }
    }
}

fn visibility(visible: bool) -> Visibility {
    match visible {
        true => Visibility::Inherited,
        false => Visibility::Hidden,
    }
}

#[derive(Debug, Component)]
pub struct DecorSprite(pub WorldTileIndex);

/// When receive tiles, we receive all decor just after.
fn on_inserted_tiles(
    tiles: On<InsertedTiles>,
    mut commands: Commands,
    existing: Query<(Entity, &Region), With<DecorSprite>>,
) {
    despawn_region(&mut commands, tiles.0, &existing);
}

fn on_remove_decor(
    remove: On<RemoveDecor>,
    mut commands: Commands,
    existing: Query<(Entity, &Region, &DecorSprite)>,
) {
    for (entity, region, sprite) in &existing {
        if region.0 == remove.0 && remove.1.contains(&sprite.0) {
            commands.entity(entity).despawn();
        }
    }
}

fn on_insert_decor(
    insert: On<InsertDecor>,
    mut commands: Commands,
    g: Res<GameConfig>,
    sprites: Option<Res<DecorSprites>>,
    visible: Res<DecorsVisible>,
) {
    let_some!(g = &g.0, return);
    let_some!(sprites = sprites, return);

    for decor in &insert.1 {
        // Distinct z per decor to avoids z-fighting
        let total = (g.w.world_width() * g.w.world_height()).max(1) as f32;
        let z = Z_DECOR + (decor.i.0 as f32 / total) * Z_DECOR_SPAN;
        commands.spawn((
            DecorSprite(decor.i),
            Region(insert.0),
            sprites.sprite(decor.tile),
            Transform::from_translation(position(decor, &sprites, &g.w).extend(z))
                .with_rotation(Quat::from_rotation_z(rotation(decor.i))),
            Anchor::CENTER,
            visibility(visible.0),
        ));
    }
}

/// Gui position of the center of the decor sprite (the rotation center): bottom of the sprite is
/// aligned on the bottom of the tile, then tileset offset is applied.
fn position(decor: &Decor, sprites: &DecorSprites, w: &oc_root::WorldConfig) -> Vec2 {
    let xy: Xy = decor.i.into_(w);
    let tile = w.geo_pixels_per_tile() as f32;
    let offset = sprites.tileset.offset;
    let size = Vec2::new(
        sprites.tileset.tile_width as f32,
        sprites.tileset.tile_height as f32,
    );
    let x = xy.0 as f32 * tile + offset.x;
    let y = (xy.1 as f32 + 1.) * tile + offset.y;
    // Bottom left to center: right, and up (y oriented to the bottom here)
    Vec2::new(x + size.x / 2., (y - size.y / 2.).to_gui_y(w))
}

fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    existing: Query<(Entity, &Region), With<DecorSprite>>,
) {
    despawn_region(&mut commands, region.0, &existing);
}

fn despawn_region(
    commands: &mut Commands,
    region: WorldRegionIndex,
    existing: &Query<(Entity, &Region), With<DecorSprite>>,
) {
    for (entity, region_) in existing {
        if region_.0 == region {
            commands.entity(entity).despawn();
        }
    }
}
