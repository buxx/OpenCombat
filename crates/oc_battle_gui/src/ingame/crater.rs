use bevy::{prelude::*, sprite::Anchor};
use oc_geo::region::WorldRegionIndex;
use oc_physics::update::bevy::Region;
use oc_root::y::Y;
use oc_utils::let_some;

use crate::{
    ingame::{draw::Z_CRATER, region::ForgottenRegion},
    sprites::crater::CraterSprites,
    states::GameConfig,
    world::{InsertCraters, InsertedTiles},
};

pub struct CraterPlugin;

impl Plugin for CraterPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_inserted_tiles)
            .add_observer(on_insert_craters)
            .add_observer(on_forgotten_region);
    }
}

#[derive(Debug, Component)]
pub struct CraterSprite;

/// When receive tiles, we receive all craters just after.
fn on_inserted_tiles(
    tiles: On<InsertedTiles>,
    mut commands: Commands,
    existing: Query<(Entity, &Region), With<CraterSprite>>,
) {
    despawn_region(&mut commands, tiles.0, &existing);
}

fn on_insert_craters(
    insert: On<InsertCraters>,
    mut commands: Commands,
    g: Res<GameConfig>,
    sprites: Option<Res<CraterSprites>>,
) {
    let_some!(g = &g.0, return);
    let_some!(sprites = sprites, return);

    for crater in &insert.1 {
        commands.spawn((
            CraterSprite,
            Region(insert.0),
            sprites.sprite(crater.type_, crater.sprite, crater.size),
            Transform::from_xyz(
                crater.position.x,
                crater.position.y.to_gui_y(&g.w),
                Z_CRATER,
            ),
            Anchor::CENTER,
        ));
    }
}

fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    existing: Query<(Entity, &Region), With<CraterSprite>>,
) {
    despawn_region(&mut commands, region.0, &existing);
}

fn despawn_region(
    commands: &mut Commands,
    region: WorldRegionIndex,
    existing: &Query<(Entity, &Region), With<CraterSprite>>,
) {
    for (entity, region_) in existing {
        if region_.0 == region {
            commands.entity(entity).despawn();
        }
    }
}
