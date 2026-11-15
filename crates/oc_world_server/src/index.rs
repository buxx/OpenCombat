use oc_utils::index::AtIndex;
use std::ops::{Deref, DerefMut};

use oc_geo::region::Region;
use oc_geo::tile::TileXy;
use oc_geo::{region::WorldRegionIndex, tile::WorldTileIndex};
use oc_individual::squad::SquadIndex;
use oc_individual::{
    INDIVIDUAL_INDEXATION_SHAPE, INDIVIDUAL_STAND_UP_VOLUME_HEIGHT,
    INDIVIDUAL_STAND_UP_VOLUME_WIDTH, IndividualIndex,
};
use oc_projectile::Projectile;
use oc_projectile::ProjectileId;
use oc_root::side::Side;
use oc_root::{WcfgFrom, WcfgInto, WorldConfig};
use oc_utils::d2::{Xy, shape_cover_tiles};
use oc_world::World;

use crate::physics;
use crate::utils::d2::shape_from_tile;

pub struct SizedIndex<T>(Vec<Vec<T>>);

impl<T: std::clone::Clone> SizedIndex<T> {
    pub fn new(size: usize) -> Self {
        Self(vec![vec![]; size])
    }
}

impl<T> Deref for SizedIndex<T> {
    type Target = Vec<Vec<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SizedIndex<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Indexes {
    /// Individual presence on tiles (considering volume)
    tiles_individuals: SizedIndex<IndividualIndex>,
    /// Same than tiles_individuals but considering individuals in larger area (for )
    proximity_individuals: SizedIndex<IndividualIndex>,
    regions_individuals: SizedIndex<IndividualIndex>,
    regions_projectiles: SizedIndex<ProjectileId>,
    regions_craters: SizedIndex<usize>,
    individuals_squad: Vec<SquadIndex>,
    side_a_individuals: Vec<IndividualIndex>,
    side_b_individuals: Vec<IndividualIndex>,
}

impl Indexes {
    pub fn new(world: &World, w: &WorldConfig) -> Self {
        let individuals = world.individuals();

        let mut tiles_individuals = SizedIndex::new(world.w.tiles_count() as usize);
        let mut proximity_individuals = SizedIndex::new(world.w.tiles_count() as usize);
        let mut regions_individuals = SizedIndex::new(world.w.regions_count() as usize);
        let mut regions_projectiles = SizedIndex::new(world.w.regions_count() as usize);
        let mut regions_craters = SizedIndex::new(world.w.regions_count() as usize);
        let mut individuals_squad = Vec::with_capacity(world.individuals.len());
        let side_a_count = individuals.iter().filter(|i| i.side == Side::A).count();
        let mut side_a_individuals = Vec::with_capacity(side_a_count);
        let side_b_count = individuals.iter().filter(|i| i.side == Side::B).count();
        let mut side_b_individuals = Vec::with_capacity(side_b_count);

        // First squad owning each member (looked up once, not by individual)
        let mut squad_of_member: Vec<Option<SquadIndex>> = vec![None; individuals.len()];
        for (squad_i, squad) in world.squads.iter().enumerate().rev() {
            for member in &squad.members {
                if let Some(slot) = squad_of_member.get_at_mut(*member) {
                    *slot = Some(SquadIndex::from(squad_i));
                }
            }
        }

        for (i, individual) in individuals.iter().enumerate() {
            let position = individual.position;
            let tile: WorldTileIndex = individual.tile;
            let region: WorldRegionIndex = tile.into_(&world.w);

            // List tiles covered by individual shape
            for tile_ in shape_cover_tiles(
                [position.x, position.y],
                INDIVIDUAL_INDEXATION_SHAPE.pixels(w),
                INDIVIDUAL_INDEXATION_SHAPE.pixels(w),
                w.geo_pixels_per_tile() as f32,
                w.geo_pixels_per_tile() as f32,
            ) {
                let tile_ = TileXy(Xy(tile_[0] as u64, tile_[1] as u64));
                if tile_.0.0 >= w.world_width() || tile_.0.1 >= w.world_height() {
                    continue;
                }
                let tile_ = WorldTileIndex::from_(tile_, w);
                tiles_individuals.at_mut(tile_).push(i.into());
            }

            // List tiles covered by "proximity" rule
            for tile_ in shape_from_tile(tile, w.proximity_individual_rayon(), w) {
                proximity_individuals.at_mut(tile_).push(i.into());
            }

            match individual.side {
                Side::A => side_a_individuals.push(IndividualIndex::from(i)),
                Side::B => side_b_individuals.push(IndividualIndex::from(i)),
            }

            regions_individuals.at_mut(region).push(i.into());

            match squad_of_member.get(i).copied().flatten() {
                Some(squad_i) => individuals_squad.push(squad_i),
                None => panic!("There is not squad owning member {i:?}"),
            }
        }

        for (id, projectile) in world.projectiles() {
            let tile: WorldTileIndex = projectile.tile();
            let region: WorldRegionIndex = tile.into_(&world.w);

            regions_projectiles.at_mut(region).push(*id);
        }

        for (i, crater) in world.craters.iter().enumerate() {
            let tile = WorldTileIndex::from_(TileXy::from_(crater.position, w), w);
            let region: WorldRegionIndex = tile.into_(&world.w);
            regions_craters.at_mut(region).push(i);
        }

        Self {
            regions_craters,
            tiles_individuals,
            proximity_individuals,
            regions_individuals,
            regions_projectiles,
            individuals_squad,
            side_a_individuals,
            side_b_individuals,
        }
    }

    /// `i` is the position of the crater in `World::craters`
    pub fn insert_crater(&mut self, i: usize, region: WorldRegionIndex) {
        self.regions_craters.at_mut(region).push(i);
    }

    pub fn insert_projectile(&mut self, id: ProjectileId, projectile: &Projectile) {
        self.update_projectile_region(id, projectile.region(), projectile.region());
    }

    pub fn remove_projectile(&mut self, id: &ProjectileId, projectile: &Projectile) {
        let region = projectile.region();
        self.regions_projectiles.at_mut(region).retain(|p| p != id);
    }

    fn update_individual_position(
        &mut self,
        i: IndividualIndex,
        now: [f32; 2],
        // TODO: maintain individual positions index to compute it internally ?
        before: [f32; 2],
        w: &WorldConfig,
    ) {
        // List tiles covered by individual shape before move
        for before in shape_cover_tiles(
            before,
            INDIVIDUAL_STAND_UP_VOLUME_WIDTH.pixels(w),
            INDIVIDUAL_STAND_UP_VOLUME_HEIGHT.pixels(w),
            w.geo_pixels_per_tile() as f32,
            w.geo_pixels_per_tile() as f32,
        ) {
            let before = TileXy(Xy(before[0] as u64, before[1] as u64));
            if before.0.0 >= w.world_width() || before.0.1 >= w.world_height() {
                continue;
            }
            let before = WorldTileIndex::from_(before, w);
            self.tiles_individuals.at_mut(before).retain(|i_| *i_ != i);
        }

        // List tiles covered by "proximity" rule after move
        for now in shape_cover_tiles(
            now,
            INDIVIDUAL_STAND_UP_VOLUME_WIDTH.pixels(w),
            INDIVIDUAL_STAND_UP_VOLUME_HEIGHT.pixels(w),
            w.geo_pixels_per_tile() as f32,
            w.geo_pixels_per_tile() as f32,
        ) {
            let now = TileXy(Xy(now[0] as u64, now[1] as u64));
            if now.0.0 >= w.world_width() || now.0.1 >= w.world_height() {
                continue;
            }
            let now = WorldTileIndex::from_(now, w);
            self.tiles_individuals.at_mut(now).push(i);
        }

        let before = TileXy::from_(before, w);
        let before = WorldTileIndex::from_(before, w);
        let now = TileXy::from_(now, w);
        let now = WorldTileIndex::from_(now, w);

        // List tiles covered by "proximity" rule before move
        for before in shape_from_tile(before, w.proximity_individual_rayon(), w) {
            self.proximity_individuals
                .at_mut(before)
                .retain(|i_| *i_ != i);
        }

        // List tiles covered by individual shape after move
        for now in shape_from_tile(now, w.proximity_individual_rayon(), w) {
            self.proximity_individuals.at_mut(now).push(i);
        }
    }

    fn update_individual_region(
        &mut self,
        i: IndividualIndex,
        now: WorldRegionIndex,
        before: WorldRegionIndex,
    ) {
        self.regions_individuals
            .at_mut(before)
            .retain(|i_| *i_ != i);
        self.regions_individuals.at_mut(now).push(i);
    }

    fn update_projectile_region(
        &mut self,
        id: ProjectileId,
        now: WorldRegionIndex,
        before: WorldRegionIndex,
    ) {
        self.regions_projectiles
            .at_mut(before)
            .retain(|id_| *id_ != id);
        self.regions_projectiles.at_mut(now).push(id);
    }

    pub fn tile_individuals(&self, tile: WorldTileIndex) -> &Vec<IndividualIndex> {
        self.tiles_individuals.at(tile)
    }

    pub fn proximity_individuals(&self, tile: WorldTileIndex) -> &Vec<IndividualIndex> {
        self.proximity_individuals.at(tile)
    }

    pub fn region_individuals(&self, region: WorldRegionIndex) -> &Vec<IndividualIndex> {
        self.regions_individuals.at(region)
    }

    pub fn region_projectiles(&self, region: WorldRegionIndex) -> &Vec<ProjectileId> {
        self.regions_projectiles.at(region)
    }

    pub fn region_craters(&self, region: WorldRegionIndex) -> &Vec<usize> {
        self.regions_craters.at(region)
    }

    pub fn individual_squad(&self, individual: IndividualIndex) -> SquadIndex {
        *self.individuals_squad.at(individual)
    }

    pub fn react(&mut self, effect: Effect, w: &WorldConfig) {
        match effect {
            Effect::Individual(i, effect) => match effect {
                IndividualEffect::Physic(effect) => match effect {
                    physics::Effect::Position { before, after } => {
                        let now = [after.x, after.y];
                        let before = [before.x, before.y];
                        self.update_individual_position(i, now, before, w)
                    }
                    physics::Effect::Tile {
                        _before: _,
                        _after: _,
                    } => {}
                    physics::Effect::Region { before, after } => {
                        self.update_individual_region(i, after, before)
                    }
                },
            },
            Effect::Projectile(i, effect) => match effect {
                ProjectileEffect::Physic(effect) => match effect {
                    physics::Effect::Position {
                        before: _,
                        after: _,
                    }
                    | physics::Effect::Tile {
                        _before: _,
                        _after: _,
                    } => {}
                    physics::Effect::Region { before, after } => {
                        self.update_projectile_region(i, after, before)
                    }
                },
            },
        }
    }

    pub fn side_a_individuals(&self) -> &[IndividualIndex] {
        &self.side_a_individuals
    }

    pub fn side_b_individuals(&self) -> &[IndividualIndex] {
        &self.side_b_individuals
    }
}

pub enum Effect {
    Individual(IndividualIndex, IndividualEffect),
    Projectile(ProjectileId, ProjectileEffect),
}

pub enum IndividualEffect {
    Physic(physics::Effect),
}

pub enum ProjectileEffect {
    Physic(physics::Effect),
}

pub trait IntoIndexEffect<T> {
    fn to_index_effect(&self, value: T) -> Effect;
}

#[cfg(test)]
mod tests {
    use ::tests::{individual::TestIndividual, world::TestWorld};
    use oc_root::{WorldConfig, geo::WorldVec3};

    use super::*;

    #[test]
    fn test_indexes_tile_individuals() {
        // Given
        let w = WorldConfig::new(2, 2);
        let individual = TestIndividual::builder()
            .position(WorldVec3::new(4., 4., 0.))
            .build()
            .make(&w);
        let world = TestWorld::builder()
            .individuals(vec![individual])
            .build()
            .make(&w);

        // When
        let indexes = Indexes::new(&world, &w);

        // Then
        let x0y0 = indexes.tile_individuals(WorldTileIndex(0));
        let x1y0 = indexes.tile_individuals(WorldTileIndex(1));
        let x0y1 = indexes.tile_individuals(WorldTileIndex(2));
        let x1y1 = indexes.tile_individuals(WorldTileIndex(3));

        assert_eq!(x0y0, &vec![IndividualIndex(0)]);
        assert_eq!(x1y0, &vec![IndividualIndex(0)]);
        assert_eq!(x0y1, &vec![IndividualIndex(0)]);
        assert_eq!(x1y1, &vec![IndividualIndex(0)]);
    }

    #[test]
    fn test_indexes_tile_individuals_removed() {
        // Given
        let w = WorldConfig::new(2, 2);
        let individual = TestIndividual::builder()
            .position(WorldVec3::new(4., 4., 0.))
            .build()
            .make(&w);
        let world = TestWorld::builder()
            .individuals(vec![individual])
            .build()
            .make(&w);
        let mut indexes = Indexes::new(&world, &w);

        // When
        indexes.update_individual_position(IndividualIndex(0), [9., 9.], [4., 4.], &w);

        // Then
        let x0y0 = indexes.tile_individuals(WorldTileIndex(0));
        let x1y0 = indexes.tile_individuals(WorldTileIndex(1));
        let x0y1 = indexes.tile_individuals(WorldTileIndex(2));
        let x1y1 = indexes.tile_individuals(WorldTileIndex(3));

        assert_eq!(x0y0, &Vec::<IndividualIndex>::new());
        assert_eq!(x1y0, &Vec::<IndividualIndex>::new());
        assert_eq!(x0y1, &Vec::<IndividualIndex>::new());
        assert_eq!(x1y1, &vec![IndividualIndex(0)]);
    }

    #[test]
    fn test_proximity_individuals() {
        // Given
        let w = WorldConfig::new(5, 5)
            .with_geo_pixels_per_tile(1)
            .with_proximity_individual_rayon(1);
        let individual = TestIndividual::builder()
            .position(WorldVec3::new(2., 2., 0.))
            .build()
            .make(&w);
        let world = TestWorld::builder()
            .individuals(vec![individual])
            .build()
            .make(&w);
        let mut indexes = Indexes::new(&world, &w);
        let before = indexes.proximity_individuals.clone();

        // When
        indexes.update_individual_position(IndividualIndex(0), [3., 3.], [2., 2.], &w);
        let after = indexes.proximity_individuals.clone();

        // Then
        assert_eq!(
            before,
            vec![
                // line 0
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                // line 1
                vec![],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![],
                // line 2
                vec![],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![],
                // line 3
                vec![],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![],
                // line 4
                vec![],
                vec![],
                vec![],
                vec![],
                vec![]
            ]
        );
        assert_eq!(
            after,
            vec![
                // line 0
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                // line 1
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                // line 2
                vec![],
                vec![],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                // line 3
                vec![],
                vec![],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                // line 4
                vec![],
                vec![],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
                vec![IndividualIndex(0)],
            ]
        );
    }
}
