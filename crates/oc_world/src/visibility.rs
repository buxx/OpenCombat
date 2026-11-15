use derive_more::{Constructor, Deref, DerefMut};
use oc_individual::IndividualIndex;
#[cfg(feature = "debug")]
use oc_root::geo::WorldVec3;
use oc_root::opacity::CumulatedOpacity;

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct Visibilities {
    /// Vector containing visibility for each individual for each individual
    values: Vec<Vec<Visibility>>,
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Visibility {
    pub visible: bool,
    pub opacity: CumulatedOpacity,
    #[cfg(feature = "debug")]
    pub sections: Vec<(WorldVec3, WorldVec3, CumulatedOpacity)>,
}

impl Default for Visibility {
    fn default() -> Self {
        Self {
            visible: false,
            opacity: CumulatedOpacity(1.0),
            #[cfg(feature = "debug")]
            sections: vec![],
        }
    }
}

impl Visibilities {
    pub fn empty(count: usize) -> Self {
        let values = vec![vec![Visibility::default(); count]; count];
        Self { values }
    }

    pub fn for_(&self, i: IndividualIndex) -> &Vec<Visibility> {
        &self.values[i.0 as usize]
    }

    #[cfg(feature = "tests")]
    pub fn values_mut(&mut self) -> &mut Vec<Vec<Visibility>> {
        &mut self.values
    }
}
