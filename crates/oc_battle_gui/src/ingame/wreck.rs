use bevy::prelude::*;
use oc_geo::region::WorldRegionIndex;
use oc_physics::update::bevy::Region;
use oc_root::{WcfgFrom, geo::ScreenVec2, y::V};
use oc_utils::let_some;
use oc_world::wreck::Wreck;

use crate::{
    ingame::{
        draw::Z_WRECK,
        region::ForgottenRegion,
        vehicle::{DESTROYED_COLOR, chassis_shadow},
    },
    sprites::vehicle::VehicleSprites,
    states::GameConfig,
    world::{InsertWrecks, InsertedTiles},
};

pub struct WreckPlugin;

impl Plugin for WreckPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_inserted_tiles)
            .add_observer(on_insert_wrecks)
            .add_observer(on_forgotten_region);
    }
}

#[derive(Debug, Component)]
pub struct WreckSprite;

/// When receive tiles, we receive all wrecks just after.
fn on_inserted_tiles(
    tiles: On<InsertedTiles>,
    mut commands: Commands,
    existing: Query<(Entity, &Region), With<WreckSprite>>,
) {
    despawn_region(&mut commands, tiles.0, &existing);
}

/// Wrecks are drawn like destroyed vehicles (darkened chassis, no turret)
fn on_insert_wrecks(
    insert: On<InsertWrecks>,
    mut commands: Commands,
    g: Res<GameConfig>,
    sprites: Option<Res<VehicleSprites>>,
) {
    let_some!(g = &g.0, return);
    let_some!(sprites = sprites, return);

    for wreck in &insert.1 {
        match wreck {
            Wreck::Vehicle {
                type_,
                position,
                chassis_direction,
            } => {
                let row = g.mod_.vehicle(*type_).row;
                let mut sprite = sprites.chassis(row);
                sprite.color = DESTROYED_COLOR;
                let screen = ScreenVec2::from_(*position, &g.w);
                let rotation = chassis_direction.bquat(V::Gui);
                commands.spawn((
                    WreckSprite,
                    Region(insert.0),
                    sprite,
                    Transform::from_xyz(screen.x, screen.y, Z_WRECK).with_rotation(rotation),
                    children![chassis_shadow(&sprites, row, rotation)],
                ));
            }
        }
    }
}

fn on_forgotten_region(
    region: On<ForgottenRegion>,
    mut commands: Commands,
    existing: Query<(Entity, &Region), With<WreckSprite>>,
) {
    despawn_region(&mut commands, region.0, &existing);
}

fn despawn_region(
    commands: &mut Commands,
    region: WorldRegionIndex,
    existing: &Query<(Entity, &Region), With<WreckSprite>>,
) {
    for (entity, region_) in existing {
        if region_.0 == region {
            commands.entity(entity).despawn();
        }
    }
}
