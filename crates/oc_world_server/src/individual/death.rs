use oc_individual::{IndividualIndex, Status, Update as IndividualUpdate, behavior::Intent};
use oc_mod::Mod;
#[cfg(feature = "debug")]
use oc_root::WorldConfig;
use oc_world::World;

use crate::runner::update::Update;

/// Updates to apply when an individual die (nothing if already dead)
pub fn die(world: &World, mod_: &Mod, i: IndividualIndex) -> Vec<Update> {
    let individual = world.individual(i);
    let mut updates = vec![];

    if matches!(individual.status, Status::Dead) {
        return updates;
    }

    #[cfg(feature = "debug")]
    let mortal = !WorldConfig::immortality();
    #[cfg(not(feature = "debug"))]
    let mortal = true;

    if mortal {
        let intent = Intent::Idle(individual.gesture.direction());
        updates.push(Update::UpdateIndividual(
            i,
            IndividualUpdate::SetStatus(Status::Dead),
        ));
        updates.push(Update::UpdateIndividual(
            i,
            IndividualUpdate::SetForces(vec![]),
        ));
        updates.push(Update::UpdateIndividual(
            i,
            IndividualUpdate::SetOrders(vec![]),
        ));
        updates.push(Update::UpdateIndividual(
            i,
            IndividualUpdate::SetIntent(intent),
        ));
    }

    if let Some(sound) = mod_.pick_die_sound() {
        updates.push(Update::PlaySoundWithGeo(
            sound,
            individual.position,
            individual.region(),
            oc_physics::fx::SoundBus::Effects,
        ));
    }

    updates
}
