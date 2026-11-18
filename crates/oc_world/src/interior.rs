use derive_more::Constructor;
use oc_root::geo::WorldVec2;
use oc_utils::polygon::Polygon;

#[cfg_attr(feature = "bevy", derive(bevy::prelude::Component))]
#[derive(Clone, Debug, Constructor)]
pub struct Interior {
    pub id: u32,
    pub start: WorldVec2,
    pub points: Vec<WorldVec2>,
    pub penetration_protect: f32,
    pub destruction_resistance: f32,
    pub destruction_alter_natures: Option<Vec<String>>,
}

impl Interior {
    pub fn penetrated(&self, position: WorldVec2, penetration_force: f32) -> bool {
        self.contains(position) && self.penetration_protect < penetration_force
    }
}

impl Polygon for Interior {
    fn points(&self) -> &[WorldVec2] {
        &self.points
    }
}
