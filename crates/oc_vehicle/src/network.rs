use crate::{Update, Vehicle as Vehicle_, VehicleIndex};

#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum Vehicle {
    Insert(VehicleIndex, Vehicle_),
    Update(VehicleIndex, Update),
    Forgot(VehicleIndex),
}
