use bevy::prelude::*;
use oc_mod::weapons::{Weapon, WeaponType};
use oc_root::files::AssetDir;
use oc_utils::bevy::EntityMapping;
use oc_utils::{let_ok, let_some};

use crate::{
    ingame::{individual::Weapons, input::projectile::InsertProjectileEvent},
    states::GameConfig,
};

const FLASH_DURATION: f32 = 0.08;
// TODO: if new weapon with barrel at other pixel, must use matching coordinates
/// Muzzle flash position relative to the individual: approximately the end of the barrel whatever the pose
pub const FLASH_X: f32 = 1.5;
pub const FLASH_Y: f32 = 16.;

#[derive(Debug, Resource)]
pub struct WeaponAnimations {
    rifle: Sprite,
    machine_gun: Sprite,
    flash: Sprite,
}

impl WeaponAnimations {
    pub fn init(sprites: &AssetDir, soldier: &Sprite, assets: &AssetServer) -> Self {
        let load = |name: &str| Sprite {
            image: assets.load(sprites.join(name)),
            texture_atlas: soldier.texture_atlas.clone(),
            ..default()
        };

        Self {
            rifle: load("weapon_rifle.png"),
            machine_gun: load("weapon_machinegun.png"),
            flash: Sprite::from_image(assets.load(sprites.join("muzzle_flash.png"))),
        }
    }

    pub fn sprite(&self, type_: WeaponType) -> Sprite {
        match type_ {
            WeaponType::Rifle => self.rifle.clone(),
            WeaponType::MachineGun => self.machine_gun.clone(),
        }
    }

    pub fn flash_sprite(&self) -> Sprite {
        self.flash.clone()
    }
}

#[derive(Debug, Component)]
pub struct IndividualWeapon;

#[derive(Debug, Component)]
pub struct IndividualMuzzleFlash(pub Timer);

/// Key of the muzzle flash entity of an individual (see `EntityMapping<MuzzleFlashOf>`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MuzzleFlashOf(pub oc_individual::IndividualIndex);

impl Default for IndividualMuzzleFlash {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(FLASH_DURATION, TimerMode::Once);
        timer.tick(timer.duration());
        Self(timer)
    }
}

pub fn sync_individual_weapons(
    g: Res<GameConfig>,
    animations: Option<Res<WeaponAnimations>>,
    individuals: Query<(&Sprite, &Weapons), Without<IndividualWeapon>>,
    mut overlays: Query<(&ChildOf, &mut Sprite, &mut Visibility), With<IndividualWeapon>>,
) {
    let_some!(g = &g.0, return);
    let_some!(animations = animations, return);

    for (parent, mut overlay, mut visibility) in &mut overlays {
        let weapons = individuals.get(parent.parent());
        let_ok!(weapons = weapons, continue);
        let (sprite, weapons) = weapons;

        // FIXME: in future, will depend about which weapon is curently used
        let type_ = weapons
            .primary
            .as_ref()
            .map(|weapon| match &g.mod_.weapon(weapon.i).1 {
                Weapon::Rifle(_) => WeaponType::Rifle,
                Weapon::MachineGun(_) => WeaponType::MachineGun,
            });

        let visibility_ = match type_ {
            Some(_) => Visibility::Inherited,
            None => Visibility::Hidden,
        };
        if *visibility != visibility_ {
            *visibility = visibility_;
        }

        if let Some(type_) = type_ {
            let image = animations.sprite(type_).image;
            if overlay.image != image {
                overlay.image = image;
            }
        }

        let index = sprite.texture_atlas.as_ref().map(|atlas| atlas.index);
        if let (Some(index), Some(atlas)) = (index, overlay.texture_atlas.as_mut())
            && atlas.index != index
        {
            atlas.index = index;
        }
    }
}

pub fn on_fire(
    projectile: On<InsertProjectileEvent>,
    mapping: Res<EntityMapping<MuzzleFlashOf>>,
    mut flashes: Query<(&mut IndividualMuzzleFlash, &mut Visibility)>,
) {
    let_some!(shooter = projectile.1.shooter(), return);
    let_some!(entity = mapping.get(&MuzzleFlashOf(shooter)), return);
    let_ok!((mut flash, mut visibility) = flashes.get_mut(*entity), return);

    flash.0.reset();
    *visibility = Visibility::Visible;
}

pub fn hide_muzzle_flashes(
    time: Res<Time>,
    mut flashes: Query<(&mut IndividualMuzzleFlash, &mut Visibility)>,
) {
    for (mut flash, mut visibility) in &mut flashes {
        if flash.0.is_finished() {
            continue;
        }
        if flash.0.tick(time.delta()).is_finished() {
            *visibility = Visibility::Hidden;
        }
    }
}
