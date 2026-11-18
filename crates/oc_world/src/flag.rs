use derive_more::Constructor;
use oc_root::geo::WorldVec2;
use oc_utils::d2::Shape;
use oc_utils::polygon::Polygon;

use crate::spawn::SpawnZone;

#[oc_derive::archivable(serde)]
#[derive(Debug, Constructor, Clone, PartialEq)]
pub struct FlagName(pub String);

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub name: FlagName,
    pub start: WorldVec2,
    pub width: f32,
    pub height: f32,
    pub corners: [WorldVec2; 4],
}

impl Flag {
    pub fn new(name: FlagName, start: WorldVec2, width: f32, height: f32) -> Self {
        let (x, y) = (start.x, start.y);
        let corners = [
            WorldVec2::new(x, y),
            WorldVec2::new(x + width, y),
            WorldVec2::new(x + width, y + height),
            WorldVec2::new(x, y + height),
        ];

        Self {
            name,
            start,
            width,
            height,
            corners,
        }
    }

    pub fn center(&self) -> WorldVec2 {
        WorldVec2::new(
            self.start.x + self.width / 2.,
            self.start.y + self.height / 2.,
        )
    }

    pub fn shape(&self) -> Shape {
        Shape {
            top_left: self.corners[0].into(),
            top_right: self.corners[1].into(),
            bottom_right: self.corners[2].into(),
            bottom_left: self.corners[3].into(),
        }
    }
}

impl Polygon for Flag {
    fn points(&self) -> &[WorldVec2] {
        &self.corners
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub enum Ownership {
    Nobody,
    A,
    B,
    Both,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Constructor, Clone, PartialEq, Default)]
pub struct Flags(Vec<(Flag, Ownership)>);

impl Flags {
    pub fn from_control(flags: Vec<Flag>, a_spawns: &[SpawnZone], b_spawns: &[SpawnZone]) -> Self {
        let mut ownerships = vec![];

        for flag in flags {
            let is_a_control = a_spawns.iter().any(|s| s.include_shape(&flag.shape()));
            let is_b_control = b_spawns.iter().any(|s| s.include_shape(&flag.shape()));

            let flag_ownership = match (is_a_control, is_b_control) {
                (true, true) => Ownership::Both,
                (true, false) => Ownership::A,
                (false, true) => Ownership::B,
                (false, false) => Ownership::Nobody,
            };
            ownerships.push((flag, flag_ownership));
        }

        Self(ownerships)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Flag, Ownership)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (Flag, Ownership)> {
        self.0.iter_mut()
    }
}
