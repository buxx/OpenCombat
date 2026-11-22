use bevy::prelude::*;
use oc_utils::let_some;

use crate::{
    entity::individual::{IndividualIndex, Side},
    ingame::individual,
    world::{UpdateVisibilities, VisibilitiesUpdated},
};

#[derive(Debug, Clone, Default)]
pub struct Visibilities {
    pub individuals: Vec<oc_individual::IndividualIndex>,
    /// Visible opposite vehicles
    pub vehicles: Vec<oc_vehicle::VehicleIndex>,
    #[cfg(feature = "debug")]
    pub all: Vec<(
        oc_individual::IndividualIndex,
        oc_individual::IndividualIndex,
        oc_world::visibility::Visibility,
    )>,
}

impl Visibilities {
    pub fn from_visibilities(
        visibilities: &[(
            oc_individual::IndividualIndex,
            oc_individual::IndividualIndex,
            oc_world::visibility::Visibility,
        )],
        vehicles: &[(
            oc_individual::IndividualIndex,
            oc_vehicle::VehicleIndex,
            oc_world::visibility::Visibility,
        )],
    ) -> Self {
        let individuals = visibilities
            .iter()
            .filter_map(|(_, i2, v)| v.visible.then_some(*i2))
            .collect::<Vec<_>>();
        let vehicles = vehicles
            .iter()
            .filter_map(|(_, i2, v)| v.visible.then_some(*i2))
            .collect::<Vec<_>>();

        #[cfg(not(feature = "debug"))]
        return Self { visible, vehicles };

        #[cfg(feature = "debug")]
        return Self {
            individuals,
            vehicles,
            all: visibilities.to_vec(),
        };
    }
}

pub fn on_update_visibilities(
    event: On<UpdateVisibilities>,
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
) {
    world.visibilities = Visibilities::from_visibilities(&event.0, &event.1);
    commands.trigger(VisibilitiesUpdated);
}

pub fn on_visibilities_updated(
    _: On<VisibilitiesUpdated>,
    mut query: Query<(&IndividualIndex, &mut Visibility, &Side)>,
    world: Res<crate::world::World>,
    network: Res<crate::network::state::State>,
    config: Res<crate::config::Config>,
) {
    let_some!(identity = &network.identity, return);

    for (i, mut visibility, side) in query.iter_mut() {
        // Change visibility only on other side individuals
        if side.0 != identity.side {
            let_some!(individual = world.individual(i.0), continue);
            let all = config.see_all_enemies;
            *visibility = individual::visibility(identity, i.0, individual, &world, all);
        }
    }
}
