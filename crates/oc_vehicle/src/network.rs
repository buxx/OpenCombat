use crate::{Vehicle as Vehicle_, VehicleIndex};

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Vehicle {
    Insert(VehicleIndex, Vehicle_),
    Forgot(VehicleIndex),
}
