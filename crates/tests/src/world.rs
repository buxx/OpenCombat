use bon::Builder;
use oc_geo::tile::WorldTileIndex;
use oc_individual::{Individual, IndividualIndex, squad::Squad};
use oc_mod::{Mod, nature::NatureIndex};
use oc_projectile::{Projectile, ProjectileId};
use oc_root::{WorldConfig, battle::BattlePhase, side::BySide};
use oc_utils::index::AtIndex;
use oc_utils::percent::Percent;
use oc_world::{
    World,
    decor::Decor,
    meta::Meta,
    navmesh::{Walls, navmesh},
    tile::{AlteredTile, Tile},
    visibility::Visibilities,
};
use rustc_hash::FxHashMap;

use crate::{squad::TestSquad, utils::workspace_path};

fn mod_() -> Mod {
    let path = workspace_path("mods/tests1");
    Mod::load(&path, None).unwrap()
}

#[derive(Debug, Builder)]
pub struct TestWorld {
    #[builder(default = mod_())]
    mod_: Mod,
    #[builder(default)]
    meta: Meta,
    #[builder(default = BattlePhase::Fight)]
    phase: BattlePhase,
    #[builder(default = oc_root::DEFAULT_FIGHT_DURATION)]
    duration: std::time::Duration,
    start: Option<std::time::Instant>,
    tiles: Option<Vec<Tile>>,
    altered_tiles: Option<Vec<AlteredTile>>,
    decor: Option<Vec<Option<Decor>>>,
    smoke: Option<Vec<(WorldTileIndex, Percent)>>,
    #[builder(default)]
    individuals: Vec<Individual>,
    squads: Option<Vec<Squad>>,
    #[builder(default)]
    projectiles: FxHashMap<ProjectileId, Projectile>,
    #[builder(default)]
    interiors: Vec<oc_world::interior::Interior>,
}

impl TestWorld {
    pub fn make(self, w: &WorldConfig) -> World {
        let mut tiles = self.tiles.unwrap_or_else(|| {
            (0..w.tiles_count())
                .map(|i| {
                    let nature = NatureIndex(0);
                    let traversability = self.mod_.nature(nature).traversability.clone();
                    Tile::new(WorldTileIndex(i), nature, 0, traversability)
                })
                .collect()
        });

        // Same as world loading: altered nature replaces tile nature
        let mut altered_tiles = vec![None; tiles.len()];
        for altered in self.altered_tiles.unwrap_or_default() {
            let i = altered.i.0 as usize;
            tiles[i].alter(&altered, &self.mod_);
            altered_tiles[i] = Some(altered);
        }

        let decor = self.decor.unwrap_or_else(|| vec![None; tiles.len()]);
        let mut smoke = vec![Percent::ZERO; tiles.len()];
        for (i, opacity) in self.smoke.unwrap_or_default() {
            *smoke.at_mut(i) = opacity;
        }
        let walls = tiles.as_walls(&self.mod_);
        let navmesh = navmesh(w, &walls);
        let squads = self.squads.unwrap_or_else(|| {
            self.individuals
                .iter()
                .enumerate()
                .map(|(i, individual)| {
                    TestSquad::builder()
                        .position(individual.position.into())
                        .members(vec![IndividualIndex::from(i)])
                        .build()
                        .make()
                })
                .collect()
        });
        let individuals_count = self.individuals.len();

        World {
            w: w.clone(),
            mod_: self.mod_,
            meta: self.meta,
            tiles,
            altered_tiles,
            decor,
            craters: vec![],
            side_spawn_zones: BySide::default(),
            spawn_zones: vec![],
            smoke,
            smoke_clouds: vec![],
            navmesh,
            individuals: self.individuals,
            visibilities: Visibilities::empty(individuals_count),
            squads,
            projectiles: self.projectiles,
            phase: self.phase,
            duration: self.duration,
            start: self.start,
            flags: Default::default(),
            supports: Default::default(),
            pending_supports: vec![],
            support_realized: Default::default(),
            interiors: self.interiors,
            places: vec![],
            uuids: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use oc_root::material::MaterialKind;

    use super::*;

    fn altered(mod_: &Mod) -> AlteredTile {
        let nature = mod_.nature_index("BigRock").unwrap(); // denies individuals
        AlteredTile::new(WorldTileIndex(1), nature)
    }

    fn assert_altered(world: &World) {
        let altered = altered(world.mod_());
        let tile = world.tile(altered.i).unwrap();
        assert_eq!(tile.nature, altered.nature);
        assert!(tile.allow.deny(MaterialKind::Individual));
        assert_eq!(world.altered_tiles[1], Some(altered));
        assert_eq!(world.altered_tiles[0], None);
    }

    #[test]
    fn test_altered_tiles_applied_at_build() {
        let altered = altered(&mod_());
        let world = TestWorld::builder().altered_tiles(vec![altered]);
        assert_altered(&world.build().make(&WorldConfig::new(2, 2)));
    }

    #[test]
    fn test_smoke_applied_at_build() {
        let world = TestWorld::builder().smoke(vec![(WorldTileIndex(1), Percent(80))]);
        let world = world.build().make(&WorldConfig::new(2, 2));
        assert_eq!(world.smoke_at(WorldTileIndex(1)), Percent(80));
        assert_eq!(world.smoke_at(WorldTileIndex(0)), Percent::ZERO);
    }

    #[test]
    fn test_alter_tiles() {
        let mut world = TestWorld::builder().build().make(&WorldConfig::new(2, 2));
        world.alter_tiles(vec![altered(world.mod_())]);
        assert_altered(&world);
    }
}
