// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Context;
use derive_more::Constructor;
use flate2::{Compression, write::GzEncoder};
use oc_root::{
    alter::Alter,
    files::{self, Roots},
    physics::Meters,
};
use tar::Builder;
use thiserror::Error;

use crate::{
    ammunition::{Ammunition, AmmunitionIndex, IndexedAmmunition},
    grenade::{GrenadeIndex, IndexedGrenade},
    illustration::{IllustrationIndex, IllustrationKind, IndexedIllustration},
    magazine::{IndexedMagazine, MagazineIndex},
    nature::{Nature, NatureIndex},
    shell::{IndexedShell, ShellIndex},
    sound::{IndexedSound, SoundIndex},
    weapons::{IndexedWeapon, WeaponIndex},
};

#[macro_use]
pub mod indexed;
pub mod ammunition;
pub mod armament;
pub mod grenade;
pub mod illustration;
pub mod magazine;
pub mod nature;
pub mod shell;
pub mod sound;
pub mod support;
pub mod weapons;

pub const MOD_RON: &str = "mod.ron";

pub const DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS: Meters = Meters(1.5);

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Constructor)]
#[allow(clippy::too_many_arguments)]
pub struct Mod {
    name: String,
    version: u32,
    a_name: String,
    b_name: String,
    a_color: [u8; 3],
    b_color: [u8; 3],
    #[serde(skip, default)]
    pub natures: Vec<nature::IndexedNature>,
    /// Matching NatureIndex of `Alter`
    #[serde(skip, default)]
    pub alter_natures: [nature::NatureIndex; Alter::COUNT],
    #[serde(skip, default)]
    pub sounds: Vec<sound::IndexedSound>,
    #[serde(default, rename = "sound_groups")]
    pub sound_groups_names: sound::SoundGroupsNames,
    #[serde(skip, default)]
    pub sound_groups: sound::SoundGroups,
    #[serde(skip, default)]
    pub ammunitions: Vec<ammunition::IndexedAmmunition>,
    #[serde(skip, default)]
    pub magazines: Vec<magazine::IndexedMagazine>,
    #[serde(skip, default)]
    pub weapons: Vec<weapons::IndexedWeapon>,
    #[serde(skip, default)]
    pub illustrations: illustration::IndexedIllustrations,
    #[serde(skip, default)]
    pub shells: Vec<shell::IndexedShell>,
    #[serde(skip, default)]
    pub grenades: Vec<grenade::IndexedGrenade>,
    #[serde(skip, default)]
    pub support: support::Support,
    // Below game specs
    #[serde(default = "default_human_default_stand_up_fire_meters")]
    pub human_default_stand_up_fire_meters: f32,
}

impl Mod {
    pub fn load(path: &PathBuf, cache_: Option<&Roots>) -> Result<Self, Error> {
        let mut mod_ = load_mod(path)?;

        mod_.natures = nature::load(path)?;
        mod_.alter_natures = nature::alter_natures(&mod_.natures)?;
        mod_.sounds = sound::load(path)?;
        mod_.sound_groups = mod_.resolve_sound_groups()?;
        mod_.ammunitions = ammunition::load(path)?;
        mod_.magazines = magazine::load(path)?;
        mod_.weapons = weapons::load(path, &mod_)?;
        mod_.illustrations = illustration::load(path)?;
        mod_.shells = shell::load(path, &mod_)?;
        mod_.grenades = grenade::load(path, &mod_)?;
        mod_.support = mod_.resolve_support(support::load(path)?)?;

        // TODO: centralize caching at server startup
        if let Some(cache_) = cache_ {
            cache(&mod_, path, cache_)?;
        }

        Ok(mod_)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn a_name(&self) -> &str {
        &self.a_name
    }

    pub fn b_name(&self) -> &str {
        &self.b_name
    }

    pub fn canonical(&self) -> String {
        format!("{}_{}", self.name, self.version)
    }

    pub fn archive(&self) -> String {
        format!("{}.tar.gz", self.canonical())
    }

    pub fn ammunitions_from_model_names(&self, models: &Vec<String>) -> Vec<&IndexedAmmunition> {
        let mut ammunitions: HashSet<&IndexedAmmunition> = HashSet::default();

        for model in models {
            for ammunition_ in self
                .ammunitions
                .iter()
                .filter(|a| a.model() == model)
                .collect::<Vec<_>>()
            {
                ammunitions.insert(ammunition_);
            }
        }

        ammunitions.into_iter().collect()
    }

    // fn amunitions_from_names(
    //     &self,
    //     amunitions: Vec<String>,
    // ) -> Result<Vec<&IndexedAmmunition>, Error> {
    //     amunitions
    //         .iter()
    //         .map(|amunition| {
    //             self.ammunitions
    //                 .iter()
    //                 .find(|a| a.name() == amunition)
    //                 .ok_or(Error::UnknownAmunitionName(amunition.clone()))
    //         })
    //         .collect::<Result<Vec<&IndexedAmmunition>, Error>>()
    // }

    pub fn ammunition_from_name(&self, name: &str) -> Result<&IndexedAmmunition, Error> {
        self.ammunitions
            .iter()
            .find(|a| a.name() == name)
            .ok_or(Error::UnknownAmmunitionName(name.to_string()))
    }

    pub fn shell_from_name(&self, name: &str) -> Result<&IndexedShell, Error> {
        self.shells
            .iter()
            .find(|s| s.name == name)
            .ok_or(Error::UnknownShellName(name.to_string()))
    }

    pub fn grenade_from_name(&self, name: &str) -> Result<&IndexedGrenade, Error> {
        self.grenades
            .iter()
            .find(|g| g.name == name)
            .ok_or(Error::UnknownGrenadeName(name.to_string()))
    }

    pub fn grenade(&self, index: GrenadeIndex) -> &IndexedGrenade {
        &self.grenades[index.0 as usize]
    }

    pub fn shell(&self, index: ShellIndex) -> &IndexedShell {
        &self.shells[index.0 as usize]
    }

    fn resolve_support(&self, raw: support::SupportRaw) -> Result<support::Support, Error> {
        Ok(support::Support {
            naval_bombard_shell: self.shell_from_name(&raw.naval_bombard_shell)?.index(),
            naval_bombard_series: raw.naval_bombard_series,
            naval_bombard_shells_by_serie: raw.naval_bombard_shells_by_serie,
            naval_bombard_radius: raw.naval_bombard_radius,
            naval_bombard_prepare_seconds: raw.naval_bombard_prepare_seconds,
            naval_bombard_serie_interval_ms: raw.naval_bombard_serie_interval_ms,
            naval_bombard_shell_interval_ms: raw.naval_bombard_shell_interval_ms,
            naval_bombard_between_wait_s: raw.naval_bombard_between_wait_s,
        })
    }

    pub fn weapon_from_name(&self, name: &str) -> Result<&IndexedWeapon, Error> {
        self.weapons
            .iter()
            .find(|w| w.name() == name)
            .ok_or(Error::UnknownWeaponName(name.to_string()))
    }

    fn magazines_from_names(&self, magazines: Vec<String>) -> Result<Vec<&IndexedMagazine>, Error> {
        magazines
            .iter()
            .map(|magazine| {
                self.magazines
                    .iter()
                    .find(|a| a.name() == magazine)
                    .ok_or(Error::UnknownMagazineName(magazine.clone()))
            })
            .collect::<Result<Vec<&IndexedMagazine>, Error>>()
    }

    pub fn magazine_from_name(&self, name: &str) -> Result<&IndexedMagazine, Error> {
        self.magazines
            .iter()
            .find(|m| m.name() == name)
            .ok_or(Error::UnknownMagazineName(name.to_string()))
    }

    pub fn ammunition(&self, index: AmmunitionIndex) -> &Ammunition {
        &self.ammunitions[index.0 as usize]
    }

    pub fn weapon(&self, index: WeaponIndex) -> &IndexedWeapon {
        &self.weapons[index.0 as usize]
    }

    pub fn magazine(&self, index: MagazineIndex) -> &IndexedMagazine {
        &self.magazines[index.0 as usize]
    }

    fn find_sounds(&self, sounds: &[String]) -> Result<Vec<&IndexedSound>, Error> {
        sounds
            .iter()
            .map(|name| {
                self.sounds
                    .iter()
                    .find(|s| &s.name == name)
                    .ok_or(Error::UnknownSoundName(name.clone()))
            })
            .collect()
    }

    fn find_sound_indexes(&self, names: &[String]) -> Result<Vec<SoundIndex>, Error> {
        Ok(self.find_sounds(names)?.iter().map(|s| s.index()).collect())
    }

    pub fn find_sound(&self, name: &str) -> Result<SoundIndex, Error> {
        self.sounds
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.index())
            .ok_or(Error::UnknownSoundName(name.to_string()))
    }

    fn resolve_sound_groups(&self) -> Result<sound::SoundGroups, Error> {
        let names = &self.sound_groups_names;
        let many = |names: &Vec<String>| -> Result<Vec<SoundIndex>, Error> {
            Ok(self
                .find_sounds(names)?
                .into_iter()
                .map(|s| s.index())
                .collect())
        };
        let one = |name: &Option<String>| -> Result<Option<SoundIndex>, Error> {
            name.as_deref()
                .map(|name| self.find_sound(name))
                .transpose()
        };

        Ok(sound::SoundGroups {
            die: many(&names.die)?,
            main_click: one(&names.main_click)?,
            secondary_click: one(&names.secondary_click)?,
            deployment_start: one(&names.deployment_start)?,
            fight_start: one(&names.fight_start)?,
            bullet_impact_ground: many(&names.bullet_impact_ground)?,
            bullet_wall_impact: many(&names.bullet_wall_impact)?,
            bullet_metal_impact: many(&names.bullet_metal_impact)?,
            unpin_grenade: many(&names.unpin_grenade)?,
        })
    }

    /// Pick randomly a bullet impact sound according to impacted nature
    pub fn pick_impact_sound(&self, kind: nature::ImpactKind) -> Option<SoundIndex> {
        let groups = &self.sound_groups;
        match kind {
            nature::ImpactKind::Ground => sound::pick(&groups.bullet_impact_ground),
            nature::ImpactKind::Wall => sound::pick(&groups.bullet_wall_impact),
            nature::ImpactKind::Metal => sound::pick(&groups.bullet_metal_impact),
        }
    }

    pub fn pick_unpin_grenade_sound(&self) -> Option<SoundIndex> {
        sound::pick(&self.sound_groups.unpin_grenade)
    }

    pub fn pick_die_sound(&self) -> Option<SoundIndex> {
        sound::pick(&self.sound_groups.die)
    }

    pub fn illustration_from_name(
        &self,
        kind: IllustrationKind,
        name: &str,
    ) -> Result<&IndexedIllustration, Error> {
        match kind {
            IllustrationKind::IngameSquad => self
                .illustrations
                .ingame_squads
                .iter()
                .find(|i| i.name == name)
                .ok_or(Error::UnknownIllustrationName(name.to_string())),
            IllustrationKind::IngameIndividual => self
                .illustrations
                .ingame_individuals
                .iter()
                .find(|i| i.name == name)
                .ok_or(Error::UnknownIllustrationName(name.to_string())),
        }
    }

    pub fn illustration(
        &self,
        kind: IllustrationKind,
        index: IllustrationIndex,
    ) -> &IndexedIllustration {
        match kind {
            IllustrationKind::IngameSquad => &self.illustrations.ingame_squads[index.0 as usize],
            IllustrationKind::IngameIndividual => {
                &self.illustrations.ingame_individuals[index.0 as usize]
            }
        }
    }

    pub fn sound(&self, sound: sound::SoundIndex) -> &IndexedSound {
        &self.sounds[sound.0 as usize]
    }

    pub fn nature(&self, index: NatureIndex) -> &Nature {
        &self.natures[index.0 as usize]
    }

    pub fn alter_nature(&self, alter: Alter) -> NatureIndex {
        self.alter_natures[alter.index()]
    }

    pub fn nature_index(&self, name: &str) -> Option<NatureIndex> {
        self.natures
            .iter()
            .find(|nature| nature.name == name)
            .map(|nature| nature.index())
    }
}

fn load_mod(path: &Path) -> Result<Mod, ModError> {
    let path = path.join(MOD_RON);
    let mod_ = std::fs::read_to_string(&path);
    let mod_ = mod_.context(format!("Read {}", path.display()))?;
    Ok(ron::from_str(&mod_)?)
}

// TODO: centralize caching at server startup
fn cache(mod_: &Mod, path: &PathBuf, roots: &Roots) -> Result<(), CacheError> {
    let files = files::Files::derived_only(roots.clone(), &mod_.canonical(), "");
    let cache = files.fs(files::File::ModArchive);
    if let Some(parent) = cache.parent() {
        std::fs::create_dir_all(parent).context(format!("Create dir {}", parent.display()))?;
    }
    if !std::fs::exists(&cache).context(format!("Test if {} exists", cache.display()))? {
        tracing::info!("Caching {} to {}", &mod_.name, cache.display());

        let file = std::fs::File::create(&cache);
        let file = file.context(format!("Create file {}", cache.display()))?;
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);

        // Archive content at its root: clients extract it under their own mod directory
        builder.append_dir_all(".", path).context(format!(
            "Create archive from '{}' ({})",
            path.display(),
            mod_.name
        ))?;
        builder
            .finish()
            .context(format!("Finish builder ({})", path.display()))?;
    }

    Ok(())
}

pub trait PickSound<S> {
    fn pick_sound(&self, specs: S) -> Option<sound::SoundIndex>;
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Caching: {0}")]
    Cache(#[from] CacheError),
    #[error("mod.ron: {0}")]
    Mod(#[from] ModError),
    #[error("Amunitions: {0}")]
    Amunitions(#[from] ammunition::Error),
    #[error("Unknown amunitions name: {0}")]
    Magazine(#[from] magazine::Error),
    #[error("Unknown amunitions name: {0}")]
    UnknownAmunitionName(String),
    #[error("Unknown ammunition name: {0}")]
    UnknownAmmunitionName(String),
    #[error("Unknown weapon name: {0}")]
    UnknownWeaponName(String),
    #[error("Unknown magazine name: {0}")]
    UnknownMagazineName(String),
    #[error("Unknown sound name: {0}")]
    UnknownSoundName(String),
    #[error("Unknown shell name: {0}")]
    UnknownShellName(String),
    #[error("Unknown grenade name: {0}")]
    UnknownGrenadeName(String),
    #[error("Grenades: {0}")]
    Grenades(#[from] grenade::Error),
    #[error("Shells: {0}")]
    Shells(#[from] shell::Error),
    #[error("Support: {0}")]
    Support(#[from] support::Error),
    #[error("Unknown illustration name: {0}")]
    UnknownIllustrationName(String),
    #[error("Illustrations: {0}")]
    Illustrations(#[from] illustration::Error),
    #[error("Weapons: {0}")]
    Natures(#[from] nature::Error),
    #[error("Natures: {0}")]
    Weapons(#[from] weapons::Error),
    #[error("Sounds: {0}")]
    Sounds(#[from] sound::Error),
}

#[derive(Debug, Error)]
pub enum ModError {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
}

fn default_human_default_stand_up_fire_meters() -> f32 {
    DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std1_sound_groups() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/std1");
        let mod_ = Mod::load(&path, None).unwrap();
        let groups = &mod_.sound_groups;

        assert_eq!(groups.die.len(), 13);
        assert_eq!(groups.unpin_grenade.len(), 2);
        assert!(mod_.pick_unpin_grenade_sound().is_some());
        assert!(groups.main_click.is_some());
        assert!(groups.secondary_click.is_some());
        assert!(groups.deployment_start.is_some());
        assert!(groups.fight_start.is_some());
        for kind in [
            nature::ImpactKind::Ground,
            nature::ImpactKind::Wall,
            nature::ImpactKind::Metal,
        ] {
            assert!(mod_.pick_impact_sound(kind).is_some());
        }
        assert!(mod_.weapons.iter().all(|w| w.reload_sound().is_some()));
    }
}
