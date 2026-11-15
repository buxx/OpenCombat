use oc_utils::index::AtIndex;
use std::path::Path;

use oc_root::{
    WorldConfig,
    physics::{MetersSeconds, Seconds},
};

use strum_macros::EnumIter;
use thiserror::Error;

use crate::{
    Mod, PickSound,
    ammunition::IndexedAmmunition,
    armament::{IndexedShotMode, ShotMode, ShotModeIndex, ShotModeRaw},
    indexed::{ArchivedDebug, Indexed, index_all, read_ron},
    magazine::IndexedMagazine,
    sound::SoundIndex,
};

pub const WEAPONS_RON: &str = "weapons.ron";
pub const DEFAULT_RELOAD_SOUND: &str = "ReloadGeneric1.ogg";

oc_utils::index_type! {
    pub struct WeaponIndex(pub u32);
}

pub type IndexedWeapon = Indexed<WeaponIndex, Weapon>;

/// Kind of weapon, wrapping its data `G` (the same for each kind)
#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub enum WeaponKind<G: ArchivedDebug> {
    Rifle(G),
    MachineGun(G),
}

pub type Weapon = WeaponKind<Gun>;

/// Weapon as written in weapons.ron (references by name)
pub type WeaponRaw = WeaponKind<GunRaw>;

impl<G: ArchivedDebug> WeaponKind<G> {
    pub fn gun(&self) -> &G {
        match self {
            Self::Rifle(gun) | Self::MachineGun(gun) => gun,
        }
    }

    fn try_map<H: ArchivedDebug, E>(
        self,
        f: impl FnOnce(G) -> Result<H, E>,
    ) -> Result<WeaponKind<H>, E> {
        Ok(match self {
            Self::Rifle(gun) => WeaponKind::Rifle(f(gun)?),
            Self::MachineGun(gun) => WeaponKind::MachineGun(f(gun)?),
        })
    }
}

impl Weapon {
    pub fn name(&self) -> &str {
        &self.gun().name
    }

    // TODO: derive (idem amunitions)
    pub fn is_type(&self, type_: WeaponType) -> bool {
        match self {
            Weapon::Rifle(_) => matches!(type_, WeaponType::Rifle),
            Weapon::MachineGun(_) => matches!(type_, WeaponType::MachineGun),
        }
    }

    pub fn ammunitions(&self) -> &Vec<IndexedAmmunition> {
        &self.gun().amunitions
    }

    pub fn magazines(&self) -> &Vec<IndexedMagazine> {
        &self.gun().magazines
    }

    pub fn interval(&self) -> Seconds {
        self.gun().interval
    }

    pub fn shots(&self) -> &Vec<IndexedShotMode> {
        &self.gun().shots
    }

    pub fn velocity(&self) -> MetersSeconds {
        self.gun().velocity
    }

    pub fn shot(&self, index: ShotModeIndex) -> &ShotMode {
        self.gun().shots.at(index)
    }

    pub fn reload(&self, w: &WorldConfig) -> Seconds {
        self.gun().reload / w.speed()
    }

    pub fn reload_sound(&self) -> Option<SoundIndex> {
        self.gun().reload_sound
    }

    pub fn aim(&self, w: &WorldConfig) -> Seconds {
        self.gun().aim / w.speed()
    }

    pub fn inaccuracy(&self, _w: &oc_root::WorldConfig) -> f32 {
        self.gun().inaccuracy
    }
}

// TODO: derive (idem amunitions)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumIter, Default)]
pub enum WeaponType {
    #[default]
    Rifle,
    MachineGun,
}

impl WeaponType {
    pub fn name(&self) -> &str {
        match self {
            WeaponType::Rifle => "Rifle",
            WeaponType::MachineGun => "Machine gun",
        }
    }
}

/// Data of a weapon. Generic over references to other mod items: by name in the `.ron` file
/// (`GunRaw`), resolved to indexed items once the mod is loaded (`Gun`).
#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Gun<
    A: ArchivedDebug = Vec<IndexedAmmunition>,
    M: ArchivedDebug = Vec<IndexedMagazine>,
    S: ArchivedDebug = Vec<IndexedShotMode>,
    R: ArchivedDebug = Option<SoundIndex>,
> {
    name: String,
    amunitions: A,
    magazines: M,
    shots: S,
    interval: Seconds,
    velocity: MetersSeconds,
    reload: Seconds,
    #[serde(default)]
    reload_sound: R,
    aim: Seconds,
    inaccuracy: f32,
}

pub type GunRaw = Gun<Vec<String>, Vec<String>, Vec<ShotModeRaw>, Option<String>>;

impl<A: ArchivedDebug, M: ArchivedDebug, S: ArchivedDebug, R: ArchivedDebug> Gun<A, M, S, R> {
    /// Replace the references to other mod items
    fn with_refs<A2, M2, S2, R2>(self, a: A2, m: M2, s: S2, r: R2) -> Gun<A2, M2, S2, R2>
    where
        A2: ArchivedDebug,
        M2: ArchivedDebug,
        S2: ArchivedDebug,
        R2: ArchivedDebug,
    {
        Gun {
            name: self.name,
            amunitions: a,
            magazines: m,
            shots: s,
            interval: self.interval,
            velocity: self.velocity,
            reload: self.reload,
            reload_sound: r,
            aim: self.aim,
            inaccuracy: self.inaccuracy,
        }
    }
}

impl GunRaw {
    fn resolve(self, mod_: &Mod) -> Result<Gun, Error> {
        let name = &self.name;
        let amunitions = mod_
            .ammunitions_from_model_names(&self.amunitions)
            .into_iter()
            .cloned()
            .collect();
        let magazines = mod_
            .magazines_from_names(self.magazines.clone())
            .map_err(|e| Error::MagazineRef(name.clone(), e.to_string()))?
            .into_iter()
            .cloned()
            .collect();
        let shots = self
            .shots
            .iter()
            .map(|mode| mode.clone().resolve_(mod_))
            .collect::<Result<Vec<ShotMode>, super::Error>>()
            .map_err(|e| Error::ShotMode(name.clone(), e.to_string()))?;

        // Explicit reload sound must exist, but the default one is optional
        let reload_sound = match &self.reload_sound {
            Some(sound) => Some(
                mod_.find_sound(sound)
                    .map_err(|e| Error::ReloadSound(name.clone(), e.to_string()))?,
            ),
            None => mod_.find_sound(DEFAULT_RELOAD_SOUND).ok(),
        };

        Ok(self.with_refs(amunitions, magazines, index_all(shots), reload_sound))
    }
}

pub fn load(path: &Path, mod_: &Mod) -> Result<Vec<IndexedWeapon>, Error> {
    let weapons: Vec<WeaponRaw> = read_ron::<_, Error>(path, WEAPONS_RON)?;
    let weapons = weapons
        .into_iter()
        .map(|weapon| weapon.try_map(|gun| gun.resolve(mod_)))
        .collect::<Result<Vec<Weapon>, Error>>()?;

    if weapons.is_empty() {
        return Err(Error::Empty);
    }

    Ok(index_all(weapons))
}

impl PickSound<(WeaponIndex, ShotModeIndex)> for Mod {
    fn pick_sound(&self, (weapon, shot): (WeaponIndex, ShotModeIndex)) -> Option<SoundIndex> {
        let weapon = self.weapon(weapon);
        let shot = weapon.shots().at(shot);
        let sounds = shot.sounds();
        if sounds.is_empty() {
            return None;
        }
        let i = fastrand::usize(..sounds.len());
        sounds.get(i).cloned()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    InvalidFormat(String),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("No weapons defined (require at least one)")]
    Empty,
    #[error("Amunition error ({0}): {1}")]
    Amunition(String, String),
    #[error("Amunition error ({0}): {1}")]
    AmunitionRef(String, String),
    #[error("Magazine error ({0}): {1}")]
    MagazineRef(String, String),
    #[error("Reload sound error ({0}): {1}")]
    ReloadSound(String, String),
    #[error("Invalid shot mode ({0}): {1}")]
    ShotMode(String, String),
}
