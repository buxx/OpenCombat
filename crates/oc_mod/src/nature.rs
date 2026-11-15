use std::path::Path;

use crate::indexed::{Indexed, index_all, read_ron};
use oc_root::{
    WorldConfig, alter::Alter, craters::CraterType, material::MaterialKind, opacity::Opacity,
    physics::Meters,
};
use strum::IntoEnumIterator;
use thiserror::Error;

pub const NATURES_RON: &str = "natures.ron";

oc_utils::index_type! {
    pub struct NatureIndex(pub u16);
}

pub type IndexedNature = Indexed<NatureIndex, Nature>;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub enum Traversable {
    All,
    None,
}

impl Traversable {
    pub fn can_individual(&self) -> bool {
        matches!(self, Self::All)
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementForce {
    None,
    Walk,
    Run,
    Sneak,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndividualMovement {
    #[serde(default = "default_movement_force")]
    pub force: MovementForce,
    #[serde(default = "default_movement_factor")]
    pub factor: f32,
}

impl Default for IndividualMovement {
    fn default() -> Self {
        Self {
            force: default_movement_force(),
            factor: default_movement_factor(),
        }
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Movement {
    #[serde(default)]
    pub individual: IndividualMovement,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interception {
    Never,
    Probability(f32),
}

impl Interception {
    pub fn probability(&self) -> f32 {
        match self {
            Interception::Never => 0.0,
            Interception::Probability(p) => p.clamp(0.0, 1.0),
        }
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Intercept {
    #[serde(default = "default_interception")]
    pub projectile: Interception,
}

impl Default for Intercept {
    fn default() -> Self {
        Self {
            projectile: default_interception(),
        }
    }
}

/// Kind of bullet impact on a nature (select impact sounds)
#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ImpactKind {
    #[default]
    Ground,
    Wall,
    Metal,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct NatureRaw {
    pub name: String,
    pub opacity: f32,
    pub z: Meters,
    pub deny: Vec<MaterialKind>,
    #[serde(default = "default_crawling_opacity_bonus")]
    pub crawling_opacity_bonus: f32,
    #[serde(default = "default_hiding_opacity_bonus")]
    pub hiding_opacity_bonus: f32,
    #[serde(default)]
    pub movement: Movement,
    #[serde(default)]
    pub intercept: Intercept,
    #[serde(default)]
    pub impact: ImpactKind,
    #[serde(default)]
    /// Name of the nature this one becomes when destroyed
    pub destruction_alter: Option<String>,
    #[serde(default = "default_destruction_resistance")]
    pub destruction_resistance: f32,
    #[serde(default)]
    /// Names of the alterations (like debris)
    pub alters: Vec<String>,
    /// Kind of crater
    #[serde(deserialize_with = "required")]
    pub crater: Option<CraterType>,
}

/// Permit to required `None` value explicitely in .ron file
fn required<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    <Option<T> as serde::Deserialize>::deserialize(deserializer)
}

impl From<NatureRaw> for Nature {
    fn from(value: NatureRaw) -> Self {
        Self {
            name: value.name.clone(),
            opacity: value.opacity,
            z: value.z,
            traversability: Traversability::from_deny_list(value.deny),
            crawling_opacity_bonus: value.crawling_opacity_bonus,
            hiding_opacity_bonus: value.hiding_opacity_bonus,
            movement: value.movement,
            intercept: value.intercept,
            impact: value.impact,
            destruction_alter: None,
            destruction_resistance: value.destruction_resistance,
            alters: vec![],
            crater: value.crater,
        }
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Nature {
    pub name: String,
    pub opacity: f32,
    pub z: Meters,
    pub traversability: Traversability,
    pub crawling_opacity_bonus: f32,
    pub hiding_opacity_bonus: f32,
    pub movement: Movement,
    pub intercept: Intercept,
    pub impact: ImpactKind,
    /// When tile destroyed, insert alter tile of this nature
    pub destruction_alter: Option<NatureIndex>,
    /// When explosion, alter tiles in destruction_rayon and destruction_resistance
    /// inferior to explosion destruction_force
    pub destruction_resistance: f32,
    /// When AlternateTile exist, pick from its Nature `alters` values to GUI display,
    pub alters: Vec<Alter>,
    /// Kind of crater an explosion leaves on this nature
    pub crater: Option<CraterType>,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Traversability {
    pub individual: bool,
    pub projectile: bool,
}

impl Traversability {
    fn from_deny_list(value: Vec<MaterialKind>) -> Self {
        Self {
            individual: !value.contains(&MaterialKind::Individual),
            projectile: !value.contains(&MaterialKind::Projectile),
        }
    }

    pub const fn all() -> Self {
        Self {
            individual: true,
            projectile: true,
        }
    }

    pub const fn none() -> Self {
        Self {
            individual: false,
            projectile: false,
        }
    }

    pub fn allow(&self, kind: MaterialKind) -> bool {
        match kind {
            MaterialKind::Individual => self.individual,
            MaterialKind::Projectile => self.projectile,
        }
    }

    pub fn deny(&self, kind: MaterialKind) -> bool {
        !self.allow(kind)
    }
}

impl Nature {
    pub fn opacity(&self, w: &WorldConfig, z: f32) -> Opacity {
        // Negative means "in the ground"
        if z < 0.0 {
            return Opacity(1.0);
        }

        // Above means in the air, so no opacity
        // TODO: this should be computed once (do it al mod load ? need WorldConfig ...)
        let height = self.z.0 * w.geo_pixels_per_meters();
        if z > height {
            return Opacity(0.0);
        }

        Opacity(self.opacity)
    }
}

/// Build natures from raw ones, resolving `destruction_alter` names to indexes
fn resolve(raws: &[NatureRaw]) -> Result<Vec<Nature>, Error> {
    raws.iter()
        .map(|raw| {
            let destruction_alter = raw
                .destruction_alter
                .as_deref()
                .map(|name| {
                    let index = raws.iter().position(|n| n.name == name);
                    index.map(NatureIndex::from).ok_or_else(|| {
                        Error::InvalidFormat(format!(
                            "Nature '{}' destruction_alter refers to unknown nature '{name}'",
                            raw.name
                        ))
                    })
                })
                .transpose()?;
            let alter = raw
                .alters
                .iter()
                .map(|name| {
                    name.parse::<Alter>().map_err(|e| {
                        Error::InvalidFormat(format!("Nature '{}' alter: {e}", raw.name))
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Nature {
                destruction_alter,
                alters: alter,
                ..Nature::from(raw.clone())
            })
        })
        .collect()
}

/// Build array of IndexedNature where index match with Alter index.
pub fn alter_natures(natures: &[IndexedNature]) -> Result<[NatureIndex; Alter::COUNT], Error> {
    let mut result = [None; Alter::COUNT];

    for nature in natures {
        for alter in &nature.alters {
            result[alter.index()].get_or_insert(nature.index());
        }
    }

    let mut missing = Alter::iter().filter(|alter| result[alter.index()].is_none());
    if let Some(alter) = missing.next() {
        return Err(Error::MissingAlterNature(alter));
    }

    Ok(result.map(|index| index.unwrap_or_default()))
}

pub fn load(path: &Path) -> Result<Vec<IndexedNature>, Error> {
    let natures: Vec<NatureRaw> = read_ron::<_, Error>(path, NATURES_RON)?;

    if natures.is_empty() {
        return Err(Error::Empty);
    }

    Ok(index_all(resolve(&natures)?))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Any(#[from] anyhow::Error),
    #[error("Format: {0}")]
    InvalidFormat(String),
    #[error("Format: {0}")]
    Format(#[from] ron::de::SpannedError),
    #[error("No natures defined (require at least one)")]
    Empty,
    #[error("No nature defines alter '{0:?}'")]
    MissingAlterNature(Alter),
}

fn default_crawling_opacity_bonus() -> f32 {
    0.0
}

fn default_hiding_opacity_bonus() -> f32 {
    0.0
}

fn default_movement_force() -> MovementForce {
    MovementForce::None
}

fn default_movement_factor() -> f32 {
    1.0
}

fn default_destruction_resistance() -> f32 {
    0.0
}

fn default_interception() -> Interception {
    Interception::Never
}
