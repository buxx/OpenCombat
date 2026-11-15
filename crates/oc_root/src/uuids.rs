use derive_more::Constructor;
use uuid::Uuid;

#[oc_derive::archivable(bare)]
#[derive(Debug, Clone, Default, PartialEq, Constructor)]
pub struct Uuids {
    pub individuals: Vec<Uuid>,
}

impl Uuids {
    pub fn individual(&self, i: usize) -> &Uuid {
        &self.individuals[i]
    }
}
