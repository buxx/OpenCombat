use bon::Builder;
use oc_geo::{region::WorldRegionIndex, tile::TileXy};
use oc_individual::squad::{Squad, SquadIndex};
use oc_root::{WcfgFrom, WorldConfig};

use crate::world::World;

#[derive(Debug, Builder)]
pub struct TestWorld {
    squads: Vec<(SquadIndex, Squad)>,
}

impl TestWorld {
    pub fn make(self, w: &WorldConfig) -> World {
        let mut world = World::default();

        for (i, squad) in self.squads {
            let tile = TileXy::from_(squad.position, w);
            let region = WorldRegionIndex::from_(tile, w);
            world
                .squads
                .entry(region)
                .or_default()
                .insert(i, squad.clone());
            world.squads_refs.insert(i, region);
        }

        world
    }
}
