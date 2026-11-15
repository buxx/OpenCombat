use bevy::prelude::*;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_geo::tile::TileXy;
use oc_root::{WcfgFrom, WorldConfig};
use oc_utils::let_some;

use crate::ingame::behavior::RefreshSquadsOrdersEvent;
use crate::ingame::input::individual::UpdateSquadEvent;
use crate::states::GameConfig;
use crate::world::World;
use oc_individual::squad::Update;

pub fn on_update_squad(
    event: On<UpdateSquadEvent>,
    g: Res<GameConfig>,
    mut world: ResMut<World>,
    mut commands: Commands,
) {
    let_some!(g = &g.0, return);
    let (i, update) = (event.0, &event.1);
    let_some!(region = world.squads_refs.get(&i).cloned(), return);

    for event in update_squad(&g.w, &mut world, i, region, update) {
        event.trigger(&mut commands)
    }
}

#[derive(Debug, PartialEq)]
enum UpdateSquadEffect {
    RefreshSquadsOrders(RefreshSquadsOrdersEvent),
}

impl UpdateSquadEffect {
    pub fn trigger(self, commands: &mut Commands) {
        match self {
            UpdateSquadEffect::RefreshSquadsOrders(event) => commands.trigger(event),
        }
    }
}

fn update_squad(
    w: &WorldConfig,
    world: &mut World,
    i: oc_individual::squad::SquadIndex,
    region: WorldRegionIndex,
    update: &Update,
) -> Vec<UpdateSquadEffect> {
    tracing::trace!(name = "ingame-squad-update", i=?i, update=?update);

    // Update can have modified region of squad
    let (new_region, events) = {
        let_some!(squads = world.squads.get_mut(&region), return vec![]);
        let_some!(squad = squads.get_mut(&i), return vec![]);

        // Server will emit Update::SetOrders for an Update::SetOrderPosition
        if !matches!(update, Update::SetOrderPosition(_, _)) {
            squad.apply(update);
        }

        match update {
            Update::SetOrders(orders) => (
                None,
                vec![UpdateSquadEffect::RefreshSquadsOrders(
                    RefreshSquadsOrdersEvent(i, orders.clone()),
                )],
            ),
            Update::SetPosition(position) => {
                let now_tile = TileXy::from_(*position, w);
                let now_region = RegionXy::from_(now_tile, w);
                let now_region = WorldRegionIndex::from_(now_region, w);
                ((now_region != region).then_some(now_region), vec![])
            }
            Update::SetOrderPosition(_, _)
            | Update::SetActives(_)
            | Update::SetLeader(_)
            | Update::SetStatus(_)
            | Update::Accomplished => (None, vec![]),
        }
    };

    // If squad now in new region
    if let Some(now_region) = new_region {
        // Remove squad from ol region
        if let Some(squads) = world.squads.get_mut(&region)
            && let Some(squad) = squads.remove(&i)
        {
            // And put it in new region
            world
                .squads
                .entry(now_region)
                .or_default()
                .insert(i, squad.clone());
        }

        world.squads_refs.insert(i, now_region);
    }

    events
}

#[cfg(test)]
mod tests {
    use crate::tests::world::TestWorld;

    use super::*;
    use ::tests::squad::TestSquad;
    use oc_individual::squad::SquadIndex;
    use oc_root::{WorldConfig, geo::WorldVec2};

    #[test]
    fn test_update_squad_change_region() {
        // Given
        let w = WorldConfig::new(100, 100)
            .with_geo_pixels_per_tile(5)
            .with_region_width(10)
            .with_region_height(10);
        let squad = TestSquad::builder()
            .position(WorldVec2::new(0., 0.))
            .members(vec![oc_individual::IndividualIndex(0)])
            .build()
            .make();
        let mut world = TestWorld::builder()
            .squads(vec![(SquadIndex(0), squad)])
            .build()
            .make(&w);
        let i = SquadIndex(0);
        let region = WorldRegionIndex(0);
        let update = Update::SetPosition([50., 0.].into()); // According to pixels per tile and region size, new region will be region 1

        // When
        let _ = update_squad(&w, &mut world, i, region, &update);

        // Then
        // No more squad in region 0
        assert_eq!(
            world
                .squads
                .get(&WorldRegionIndex(0))
                .cloned()
                .unwrap_or_default()
                .keys()
                .collect::<Vec<&SquadIndex>>(),
            Vec::<&SquadIndex>::new(),
        );
        // Squad is in region 1
        assert_eq!(
            world
                .squads
                .get(&WorldRegionIndex(1))
                .cloned()
                .unwrap_or_default()
                .keys()
                .collect::<Vec<&SquadIndex>>(),
            vec![&SquadIndex(0)],
        );
        // squad_refs updated
        assert_eq!(
            world.squads_refs.get(&SquadIndex(0)),
            Some(&WorldRegionIndex(1))
        );
    }
}
