use bevy::prelude::*;
use oc_utils::let_some;

use crate::{
    entity::individual::{IndividualIndex, Side},
    ingame::individual,
    world::{UpdateVisibilities, VisibilitiesUpdated},
};

#[derive(Debug, Clone, Default)]
pub struct Visibilities {
    pub visible: Vec<oc_individual::IndividualIndex>,
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
    ) -> Self {
        let visible = visibilities
            .iter()
            .filter_map(|(_, i2, v)| v.visible.then_some(*i2))
            .collect::<Vec<_>>();

        #[cfg(not(feature = "debug"))]
        return Self { visible };

        #[cfg(feature = "debug")]
        return Self {
            visible,
            all: visibilities.to_vec(),
        };
    }
}

pub fn on_update_visibilities(
    event: On<UpdateVisibilities>,
    mut commands: Commands,
    mut world: ResMut<crate::world::World>,
) {
    world.visibilities = Visibilities::from_visibilities(&event.0);
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
