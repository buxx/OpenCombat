use std::{collections::HashSet, path::Path};

use oc_root::physics::{DegreesPerSeconds, Meters, MetersSeconds, Seconds};
use thiserror::Error;

use crate::{
    indexed::{ArchivedDebug, Indexed, read_ron},
    shell::ShellIndex,
};

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
pub struct Vehicle<S: ArchivedDebug = ShellIndex>
where
    Vec<S>: ArchivedDebug,
{
    pub name: String,
    /// Row of this vehicle in the mod `sprites/vehicles.png` spritesheet
    pub row: u8,
    /// Existing Crew role for this vehicle
    pub crew: Vec<CrewRole>,
    /// Chassis (width, height), used for physics
    pub size: (Meters, Meters),
    /// Chassis height, used for physics
    pub height: Meters,
    /// Maximum chassis translation speed
    pub speed: MetersSeconds,
    /// Chassis rotation speed
    pub chassis_rotation_speed: DegreesPerSeconds,
    /// Turret rotation speed
    pub turret_rotation_speed: DegreesPerSeconds,
    /// Duration for the loader to load the gun
    pub reload_duration: Seconds,
    /// Duration for the gunner to aim, once the turret is oriented to the target, before firing
    pub aim_duration: Seconds,
    /// Gun inaccuracy (degrees), added to other shell fire inaccuracy factors
    pub inaccuracy: f32,
    /// Chassis resistance to shell penetration
    pub penetration_resistance: f32,
    /// Shells this vehicle can fire
    pub compatible_shell: Vec<S>,
}

pub type VehicleRaw = Vehicle<String>;

pub fn load(path: &Path) -> Result<Vec<VehicleRaw>, Error> {
    let vehicles: Vec<VehicleRaw> = read_ron::<_, Error>(path, VEHICLES_RON)?;

    let mut names = HashSet::new();
    if let Some(vehicle) = vehicles.iter().find(|v| !names.insert(v.name.as_str())) {
        return Err(Error::DuplicateName(vehicle.name.clone()));
    }

    Ok(vehicles)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] oc_root::format::FileError),
    #[error("Duplicate vehicle name: {0}")]
    DuplicateName(String),
}
