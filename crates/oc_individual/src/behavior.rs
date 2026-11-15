use derive_more::{Deref, DerefMut};
use oc_root::{
    WorldConfig,
    geo::WorldVec2,
    physics::{Meters, MetersSeconds},
};
use oc_utils::d2::Direction;

use crate::IndividualIndex;

/// Maximum distance a grenade or smoke can be thrown. Further, individual moves to be near enough.
pub const THROW_MAX_DISTANCE: Meters = Meters(30.0);

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    Idle(Direction),
    MoveTo(WorldVec2, MovePath),
    MoveFastTo(WorldVec2, MovePath),
    SneakTo(WorldVec2, MovePath),
    Defend(Direction),
    Hide(Direction),
    Engage(IndividualIndex),
    Suppress(WorldVec2),
    Grenade(WorldVec2),
    Smoke(WorldVec2),
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Behavior {
    Idle(Direction),
    Walk(Direction),
    Run(Direction),
    Crawl(Direction),
    Defend(Direction),
    Hide(Direction),
    Engage(IndividualIndex),
    Suppress(WorldVec2),
    Grenade(WorldVec2),
    Smoke(WorldVec2),
}

impl Intent {
    pub fn path(&self) -> Option<(WorldVec2, &MovePath)> {
        match self {
            Intent::Idle(_)
            | Intent::Defend(_)
            | Intent::Hide(_)
            | Intent::Engage(_)
            | Intent::Suppress(_)
            | Intent::Grenade(_)
            | Intent::Smoke(_) => None,
            Intent::MoveTo(target, path)
            | Intent::MoveFastTo(target, path)
            | Intent::SneakTo(target, path) => Some((*target, path)),
        }
    }
}

impl Behavior {
    pub fn nominal_speed(&self, w: &WorldConfig) -> MetersSeconds {
        match self {
            Behavior::Idle(_)
            | Behavior::Defend(_)
            | Behavior::Hide(_)
            | Behavior::Engage(_)
            | Behavior::Suppress(_)
            | Behavior::Grenade(_)
            | Behavior::Smoke(_) => MetersSeconds(0.0),
            Behavior::Walk(_) => w.walk_nominal_speed(),
            Behavior::Run(_) => w.run_nominal_speed(),
            Behavior::Crawl(_) => w.crawl_nominal_speed(),
        }
    }
}

#[oc_derive::archivable]
#[derive(Debug, Clone, Deref, DerefMut, PartialEq)]
pub struct MovePath(pub Vec<WorldVec2>);

#[cfg(feature = "polyanya")]
impl From<polyanya::Path> for MovePath {
    fn from(value: polyanya::Path) -> Self {
        let path = value.path.iter().map(|p| [p.x, p.y].into()).collect();
        Self(path)
    }
}
