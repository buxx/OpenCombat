/// Kind of crater left on the ground by an explosion, according to the ground hardness
#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CraterType {
    Mud,
    Concrete,
}

impl CraterType {
    /// Row of this type in the sprite file
    pub const fn row(self) -> usize {
        self as usize
    }
}
