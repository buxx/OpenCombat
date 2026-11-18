use bevy::{prelude::*, sprite::Anchor};
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_physics::update::bevy::Region;
use oc_root::{WcfgInto, y::Y};
use oc_utils::{d2::Xy, let_some, true_};
use oc_world::tile::AlteredTile;

use crate::{
    ingame::{draw::Z_ALTERED_TILE, region::ForgottenRegion},
    sprites::altered::AlteredSprites,
    states::GameConfig,
    world::{InsertAlteredTiles, InsertedTiles, World},
};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_inserted_tiles)
            .add_observer(on_insert_altered_tiles)
            .add_observer(on_forgotten_region);
    }
}

/// Debris sprite displayed on an altered tile
#[derive(Debug, Component)]
pub struct AlteredTileSprite(pub WorldTileIndex);

/// Minimum distance (pixels) between two displayed alter sprites
const MIN_DISTANCE: f32 = 20.;

/// Region tiles received: display the alterations already known for this region
fn on_inserted_tiles(
    tiles: On<InsertedTiles>,
    mut commands: Commands,
    g: Res<GameConfig>,
    world: Res<World>,
    sprites: Option<Res<AlteredSprites>>,
    existing: Query<(Entity, &Region), With<AlteredTileSprite>>,
) {
    let region = tiles.0;
    despawn_region(&mut commands, region, &existing);

    let_some!(g = &g.0, return);
    let_some!(sprites = sprites, return);
    let_some!(altered = world.altered_tiles.get(&region), return);
    let tiles: Vec<&AlteredTile> = altered.values().collect();
    spawn(&mut commands, region, tiles, vec![], g, &sprites);
}

/// Alter tiles received: replace sprites of concerned tiles
fn on_insert_altered_tiles(
    insert: On<InsertAlteredTiles>,
    mut commands: Commands,
    g: Res<GameConfig>,
    sprites: Option<Res<AlteredSprites>>,
    existing: Query<(Entity, &Region, &AlteredTileSprite, &Transform)>,
) {
    let_some!(g = &g.0, return);
    let_some!(sprites = sprites, return);
    let region = insert.0;

    let mut placed = vec![];
    for (entity, region_, sprite, transform) in &existing {
        true_!(region_.0 == region, continue);

        if insert.1.iter().any(|tile| tile.i == sprite.0) {
            commands.entity(entity).despawn();
        } else {
            placed.push(transform.translation.truncate());
        }
    }

    spawn(
        &mut commands,
        region,
        insert.1.iter().collect(),
        placed,
        g,
        &sprites,
    );
}

/// Spawn sprites for tiles
fn spawn(
    commands: &mut Commands,
    region: WorldRegionIndex,
    mut tiles: Vec<&AlteredTile>,
    mut placed: Vec<Vec2>,
    g: &oc_network::GameConfig,
    sprites: &AlteredSprites,
) {
    // stable order to ensure stable display
    tiles.sort_by_key(|tile| tile.i.0);

    for tile in tiles {
        let alters = &g.mod_.nature(tile.nature).alters;
        let alter = tile.i.pick(alters, oc_root::TILE_SALT_ALTER_SPRITE);
        let_some!(alter = alter.copied(), continue);
        let position = position(tile.i, &g.w);
        if placed.iter().any(|p| p.distance(position) < MIN_DISTANCE) {
            continue;
        }

        placed.push(position);
        commands.spawn((
            AlteredTileSprite(tile.i),
            Region(region),
            sprites.sprite(alter),
            Transform::from_xyz(position.x, position.y, Z_ALTERED_TILE)
                .with_rotation(Quat::from_rotation_z(rotation(tile.i))),
            Anchor::CENTER,
        ));
    }
}

/// Gui position of the center of tile
fn position(i: WorldTileIndex, w: &oc_root::WorldConfig) -> Vec2 {
    let xy: Xy = i.into_(w);
    let half = w.geo_pixels_per_tile() as f32 / 2.;
    let x = (xy.0 * w.geo_pixels_per_tile()) as f32 + half;
    let y = (xy.1 * w.geo_pixels_per_tile()) as f32 + half;
    Vec2::new(x, y.to_gui_y(w))
}

/// Pseudo-random angle (radians) seeded by the tile index: always the same for a given tile
pub fn rotation(i: WorldTileIndex) -> f32 {
    (i.seeded(oc_root::TILE_SALT_ROTATION) as f64 / u64::MAX as f64) as f32 * std::f32::consts::TAU
}

fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    existing: Query<(Entity, &Region), With<AlteredTileSprite>>,
) {
    despawn_region(&mut commands, region.0, &existing);
}

fn despawn_region(
    commands: &mut Commands,
    region: WorldRegionIndex,
    existing: &Query<(Entity, &Region), With<AlteredTileSprite>>,
) {
    for (entity, region_) in existing {
        if region_.0 == region {
            commands.entity(entity).despawn();
        }
    }
}
