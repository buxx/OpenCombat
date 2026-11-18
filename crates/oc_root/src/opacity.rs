use derive_more::{Add, Deref};

#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, Deref, PartialEq)]
pub struct Opacity(pub f32);

#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, Deref, PartialEq, PartialOrd, Add)]
pub struct CumulatedOpacity(pub f32);

impl CumulatedOpacity {
    pub fn inaccuracy(&self, w: &crate::WorldConfig) -> f32 {
        self.0 * w.opacity_inaccuracy
    }
}
