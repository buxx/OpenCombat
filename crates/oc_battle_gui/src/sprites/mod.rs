use crate::states::GuiFiles;
use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use oc_root::files::File;
use oc_utils::let_some;

use crate::{
    ingame::GameConfigReceived,
    sprites::{
        altered::AlteredSprites, crater::CraterSprites, soldier::SoldierAnimations,
        weapon::WeaponAnimations,
    },
};

pub mod altered;
pub mod crater;
pub mod decor;
pub mod order;
pub mod outline;
pub mod soldier;
pub mod weapon;

#[derive(Debug, Default)]
pub struct Animations;

impl Plugin for Animations {
    fn build(&self, app: &mut App) {
        app.init_resource::<oc_utils::bevy::EntityMapping<weapon::MuzzleFlashOf>>()
            .add_observer(on_game_config_received)
            .add_observer(weapon::on_fire)
            .add_systems(
                Update,
                (
                    outline::sync_individual_outlines,
                    weapon::sync_individual_weapons,
                    weapon::hide_muzzle_flashes,
                ),
            );
    }
}

fn on_game_config_received(
    _: On<GameConfigReceived>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    files: Res<GuiFiles>,
    mut animations: ResMut<Assets<Animation>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let_some!(files = &files.0, return);

    // FIXME: find solution to initialize that before (dowloading)
    // Sprites are needed as soon as individuals are received, before the download step
    for file in [File::ModArchive, File::IndividualOutline] {
        if let Err(e) = crate::downloading::ensure_file(files, file) {
            tracing::error!("Unable to get {file}: {e}");
            return;
        }
    }
    let sprites = files.sprites();

    let soldier = SoldierAnimations::init(
        &sprites,
        &files.asset(oc_root::files::File::IndividualOutline),
        &assets,
        &mut animations,
        &mut atlas_layouts,
    );
    let weapons = WeaponAnimations::init(&sprites, &soldier.sprite(), &assets);
    let altereds = AlteredSprites::init(&sprites, &assets, &mut atlas_layouts);
    let craters = CraterSprites::init(&sprites, &assets, &mut atlas_layouts);

    commands.insert_resource(craters);
    commands.insert_resource(altereds);
    commands.insert_resource(soldier);
    commands.insert_resource(weapons);
}

pub trait IntoAnimation<A> {
    fn animation(&self, animations: &A) -> Handle<Animation>;
}

pub trait SpriteRect {
    fn rect(&self) -> Rect;
}

pub trait IntoSprite<T> {
    fn sprite(&self) -> T;
}

pub trait IntoIndividualSprite<T> {
    fn individual_sprite(&self) -> T;
}
