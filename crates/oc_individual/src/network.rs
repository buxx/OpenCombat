use crate::{
    Individual as Individual_, IndividualIndex, Update,
    squad::{self, SquadIndex},
};

#[oc_derive::archivable(no_compare)]
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)] // Insert is rare compared to updates: not worth boxing
pub enum Individual {
    Insert(IndividualIndex, Individual_),
    Update(IndividualIndex, Update),
    Physics(IndividualIndex, oc_physics::update::Update),
    Forgot(IndividualIndex),
}

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Squad {
    Update(SquadIndex, squad::Update),
}
