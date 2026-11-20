use bevy::prelude::*;
use oc_vehicle::{Vehicle, VehicleIndex};

#[derive(Debug, Event)]
pub struct InsertVehicleEvent(pub VehicleIndex, pub Vehicle);

#[derive(Debug, Event)]
pub struct ForgotVehicleEvent(pub VehicleIndex);
