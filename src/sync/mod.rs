use serde::{Deserialize, Serialize};

use crate::{
    entity::{soldier::Soldier, vehicle::Vehicle},
    state::shared::SharedState,
};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct StateCopy {
    soldiers: Vec<Soldier>,
    vehicles: Vec<Vehicle>,
}

impl StateCopy {
    pub fn from_state(shared_state: &SharedState) -> StateCopy {
        let mut soldiers = vec![];
        let mut vehicles = vec![];

        for soldier in shared_state.soldiers() {
            soldiers.push(Soldier::from_soldier(soldier));
        }

        for vehicle in shared_state.vehicles() {
            vehicles.push(Vehicle::from_vehicle(vehicle));
        }

        Self { soldiers, vehicles }
    }

    pub fn soldiers(&self) -> &Vec<Soldier> {
        &self.soldiers
    }

    pub fn vehicles(&self) -> &Vec<Vehicle> {
        &&self.vehicles
    }
}
