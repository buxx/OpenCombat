use derive_more::Constructor;
use oc_individual::IndividualIndex;
use oc_root::physics::Meters;
use oc_world::visibility::Visibility;

#[derive(Debug, Clone)]
pub struct Situation<'a> {
    pub visibles: Vec<Visible<'a>>,
}

impl<'a> Situation<'a> {
    pub fn imply_hide(&self) -> bool {
        !self.visibles.is_empty()
    }

    pub fn visible(&self, individual: IndividualIndex) -> Option<&Visible<'_>> {
        self.visibles.iter().find(|v| v.individual == individual)
    }
}

#[derive(Debug, Clone, Constructor)]
pub struct Visible<'a> {
    pub individual: IndividualIndex,
    pub visibility: &'a Visibility,
    pub distance: Meters,
}
