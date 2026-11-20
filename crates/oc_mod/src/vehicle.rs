use std::{collections::HashSet, path::Path};

use oc_root::physics::Meters;
use thiserror::Error;

use crate::indexed::{Indexed, index_all, read_ron};

pub const VEHICLES_RON: &str = "vehicles.ron";

oc_utils::index_type! {
    pub struct VehicleIndex(pub u16);
}

pub type IndexedVehicle = Indexed<VehicleIndex, Vehicle>;

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrewRole {
    Commander,
    Gunner,
    Loader,
    Driver,
    Operator,
    Passenger,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq)]
pub struct Vehicle {
    pub name: String,
    /// Row of this vehicle in the mod `sprites/vehicles.png` spritesheet
    pub row: u8,
    pub crew: Vec<CrewRole>,
    /// Chassis (width, height), used for physics
    pub size: (Meters, Meters),
}

pub fn load(path: &Path) -> Result<Vec<IndexedVehicle>, Error> {
    let vehicles: Vec<Vehicle> = read_ron::<_, Error>(path, VEHICLES_RON)?;

    let mut names = HashSet::new();
    if let Some(vehicle) = vehicles.iter().find(|v| !names.insert(v.name.as_str())) {
        return Err(Error::DuplicateName(vehicle.name.clone()));
    }

    Ok(index_all(vehicles))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] oc_root::format::FileError),
    #[error("Duplicate vehicle name: {0}")]
    DuplicateName(String),
}
