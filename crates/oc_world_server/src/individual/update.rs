use oc_individual::{IndividualIndex, Update, network};
use oc_network::ToClient;
use oc_utils::index::AtIndex;
use oc_world::World;

use crate::{
    index::{self, IntoIndexEffect},
    routing::Listening,
};

/// Write `update` on individual `i`. Returns messages to broadcast and effects to apply on
/// indexes (see [`crate::index::Indexes::react`]).
pub fn write(
    world: &mut World,
    update: Update,
    i: IndividualIndex,
) -> (Vec<(Listening, Vec<ToClient>)>, Vec<index::Effect>) {
    let individual = world.individuals.at_mut(i);
    let effects = individual
        .apply(&world.w, &update)
        .into_iter()
        .map(|effect| i.to_index_effect(effect))
        .collect();

    let region = individual.region;
    let update = network::Individual::Update(i, update);
    let update = ToClient::Individual(update);
    (
        vec![(Listening::Regions(vec![region]), vec![update])],
        effects,
    )
}

#[cfg(test)]
mod tests {
    use ::tests::{individual::TestIndividual, world::TestWorld};
    use oc_geo::tile::{TileXy, WorldTileIndex};
    use oc_root::{WcfgInto, WorldConfig, geo::WorldVec3};

    use super::*;
    use crate::index::Indexes;

    #[test]
    fn set_position_updates_tile_region_and_indexes() {
        // Given: an individual on tile (0, 0) of a 10x10 tiles world
        let w = WorldConfig::new(10, 10);
        let individual = TestIndividual::builder()
            .position(WorldVec3::new(4., 4., 0.))
            .build()
            .make(&w);
        let mut world = TestWorld::builder()
            .individuals(vec![individual])
            .build()
            .make(&w);
        let mut indexes = Indexes::new(&world, &w);
        let i = IndividualIndex(0);
        let tile = |x: f32, y: f32| -> WorldTileIndex {
            let tile: TileXy = WorldVec3::new(x, y, 0.).into_(&w);
            tile.into_(&w)
        };
        assert!(indexes.tile_individuals(tile(4., 4.)).contains(&i));

        // When: it is teleported (like a deployment placement)
        let update = Update::SetPosition(WorldVec3::new(42., 42., 0.));
        let (_, effects) = write(&mut world, update, i);
        for effect in effects {
            indexes.react(effect, &w);
        }

        // Then: its tile follows its position and indexes find it there only
        assert_eq!(world.individual(i).tile, tile(42., 42.));
        assert!(indexes.tile_individuals(tile(42., 42.)).contains(&i));
        assert!(!indexes.tile_individuals(tile(4., 4.)).contains(&i));
    }
}
