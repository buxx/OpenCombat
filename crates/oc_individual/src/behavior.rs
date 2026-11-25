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

    pub fn path_mut(&mut self) -> Option<&mut MovePath> {
        match self {
            Intent::Idle(_)
            | Intent::Defend(_)
            | Intent::Hide(_)
            | Intent::Engage(_)
            | Intent::Suppress(_)
            | Intent::Grenade(_)
            | Intent::Smoke(_) => None,
            Intent::MoveTo(_, path) | Intent::MoveFastTo(_, path) | Intent::SneakTo(_, path) => {
                Some(path)
            }
        }
    }

    pub fn move_step_accomplished(&mut self) {
        if let Some(path) = self.path_mut()
            && !path.is_empty()
        {
            path.remove(0);
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

    #[cfg(feature = "bevy")]
    pub fn color(&self, alpha: f32) -> bevy::color::Color {
        let (r, g, b) = match self {
            Behavior::Idle(_) => (0.5, 0.5, 0.5),
            Behavior::Walk(_) => (0.0, 0.0, 1.0),
            Behavior::Run(_) => (0.0, 0.0, 0.55),
            Behavior::Crawl(_) => (0.0, 1.0, 0.0),
            Behavior::Engage(_) => (1.0, 0.0, 0.0),
            Behavior::Suppress(_) => (0.55, 0.0, 0.0),
            Behavior::Defend(_) => (1.0, 0.65, 0.0),
            Behavior::Hide(_) => (1.0, 1.0, 0.0),
            Behavior::Grenade(_) => (1.0, 0.2, 0.0),
            Behavior::Smoke(_) => (1.0, 0.5, 0.0),
        };
        bevy::color::Color::srgba(r, g, b, alpha)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_step_accomplished() {
        let target = WorldVec2::new(2., 2.);
        let path = MovePath(vec![WorldVec2::new(1., 1.), target]);
        let mut intent = Intent::MoveTo(target, path);

        intent.move_step_accomplished();
        assert_eq!(intent.path().map(|(_, p)| p.0.clone()), Some(vec![target]));
        intent.move_step_accomplished();
        intent.move_step_accomplished(); // Empty path: nothing happens
        assert_eq!(intent.path().map(|(_, p)| p.0.len()), Some(0));

        let mut idle = Intent::Idle(Direction::default());
        idle.move_step_accomplished();
        assert!(idle.path_mut().is_none());
    }
}
