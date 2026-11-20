use bevy::prelude::*;
use oc_root::Wcfg;
use oc_utils::let_some;

use crate::ingame::input::vehicle::{ForgotVehicleEvent, InsertVehicleEvent};
use crate::ingame::region::ForgottenRegion;

// TODO: these functions (on_insert, on_forgot, on_forgotten_region) can be refactored accross other things like invididuals, projectiles, etc
pub fn on_insert_vehicle(
    insert: On<InsertVehicleEvent>,
    w: Res<Wcfg>,
    mut index: ResMut<super::World>,
) {
    let_some!(w = &w.0, return);
    tracing::trace!(name="world-vehicle-insert", i=?insert.0);
    index.insert_vehicle(w, insert.0, insert.1.clone());
}

pub fn on_forgot_vehicle(forgot: On<ForgotVehicleEvent>, mut index: ResMut<super::World>) {
    tracing::trace!(name="world-vehicle-forgot", i=?forgot.0);
    index.remove_vehicle(forgot.0);
}

pub fn on_forgotten_region(region: On<ForgottenRegion>, mut index: ResMut<super::World>) {
    tracing::trace!(name = "world-vehicle-forgotten-region", region=?region.0);
    index.remove_vehicles(region.0);
}
