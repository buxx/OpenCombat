use oc_root::{craters::CraterType, geo::WorldVec2};

/// Crater left on the ground by an explosion
#[oc_derive::archivable(no_compare)]
#[derive(Debug, Clone, PartialEq)]
pub struct Crater {
    /// Center of the crater
    pub position: WorldVec2,
    /// Type of the crater (used to choos the sprite type)
    pub type_: CraterType,
    /// Index of the sprite in the row of its type
    pub sprite: usize,
    /// Diameter in pixels
    pub size: f32,
}

impl Crater {
    pub fn new(position: WorldVec2, type_: CraterType, sprite: usize, size: f32) -> Self {
        Self {
            position,
            type_,
            sprite,
            size,
        }
    }
}
