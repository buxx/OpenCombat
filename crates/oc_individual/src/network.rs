use crate::{
    Individual as Individual_, IndividualIndex, Update,
    squad::{self, SquadIndex},
};

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
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
