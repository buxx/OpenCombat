use bevy::prelude::*;
use oc_geo::UpdateGeo;
use oc_physics::{Physic, UpdatePhysic};
use oc_root::Wcfg;
use oc_utils::let_some;

use crate::ingame::input::individual::{InsertIndividualEvent, UpdateIndividualPhysicsEvent};
use crate::ingame::region::ForgottenRegion;

pub fn on_insert_individual(
    insert: On<InsertIndividualEvent>,
    w: Res<Wcfg>,
    mut index: ResMut<super::World>,
) {
    let_some!(w = &w.0, return);
    let i = insert.0;
    let individual = &insert.1;

    tracing::trace!(name="world-individual-insert", i=?i);
    index.insert_individual(w, i, individual.clone());
}

// TODO: should be automatized (macro? derive ?)
pub fn on_forgotten_region(region: On<ForgottenRegion>, mut index: ResMut<super::World>) {
    tracing::trace!(name = "world-individual-forgotten-region", region=?region.0);
    index.remove_individuals(region.0);
}

pub fn on_update_individual_physics(
    update: On<UpdateIndividualPhysicsEvent>,
    w: Res<Wcfg>,
    mut index: ResMut<super::World>,
) {
    let_some!(w = &w.0, return);
    let (i, update) = (update.0, &update.1);

    // Here, must update index with all used values by oc_physics::step
    let refresh = {
        let_some!(individual = index.get_individual_mut(i), return);
        match update {
            oc_physics::update::Update::SetTile(tile, _) => {
                let position = individual.position(w);
                individual.set_tile(*tile);
                Some(position)
            }
            oc_physics::update::Update::SetVolumes(volumes, _) => {
                individual.set_volumes(volumes.clone());
                None
            }
            oc_physics::update::Update::SetPosition(position, _) => {
                individual.set_position(*position);
                None
            }
            oc_physics::update::Update::SetRegion(_, _)
            | oc_physics::update::Update::PushForce(_)
            | oc_physics::update::Update::RemoveForce(_) => None,
        }
    };

    // TODO: Its dirty
    if let Some(position) = refresh {
        index.refresh_individual_position(w, i, position);
    }
}
