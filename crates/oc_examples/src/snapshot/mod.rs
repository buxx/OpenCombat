use oc_root::format::{FileError, RkyvFile};
use std::{marker::PhantomData, path::PathBuf};

use oc_individual::{Individual, squad::Squad};
use oc_mod::Mod;
use oc_projectile::Projectile;
use oc_root::{WorldConfig, battle::BattlePhase, side::BySide};
use oc_world::{
    snapshot::{Snapshot, State},
    spawn::SpawnZoneName,
    support::Supports,
    tile::{AlteredTile, Tile},
};

pub mod individual;
pub mod projectile;
pub mod squad;
pub mod tile;

pub struct SnapshotBuilder<T, I, S, P>
where
    T: tile::TilesGenerator,
    I: individual::IndividualsGenerator,
    S: squad::SquadsGenerator,
    P: projectile::ProjectilesGenerator,
{
    tiles: T,
    altered_tiles: Option<Vec<Option<AlteredTile>>>,
    individuals: I,
    squads: S,
    projectiles: P,
    vehicles: Vec<oc_vehicle::Vehicle>,
    phase: BattlePhase,
    side_a_spawns: Vec<SpawnZoneName>,
    side_b_spawns: Vec<SpawnZoneName>,
    supports: Supports,
    persisted: Option<PathBuf>,
}

impl<T, I, S, P> SnapshotBuilder<T, I, S, P>
where
    T: tile::TilesGenerator,
    I: individual::IndividualsGenerator,
    S: squad::SquadsGenerator,
    P: projectile::ProjectilesGenerator,
{
    pub fn new(
        tiles: T,
        individuals: I,
        squads: S,
        projectiles: P,
        phase: BattlePhase,
        side_a_spawns: Vec<SpawnZoneName>,
        side_b_spawns: Vec<SpawnZoneName>,
    ) -> Self {
        Self {
            tiles,
            individuals,
            squads,
            projectiles,
            vehicles: vec![],
            phase,
            side_a_spawns,
            side_b_spawns,
            supports: Supports::default(),
            altered_tiles: None,
            persisted: None,
        }
    }

    pub fn with_phase(mut self, phase: BattlePhase) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_supports(mut self, supports: Supports) -> Self {
        self.supports = supports;
        self
    }

    pub fn with_vehicles(mut self, vehicles: Vec<oc_vehicle::Vehicle>) -> Self {
        self.vehicles = vehicles;
        self
    }

    pub fn with_altered_tiles(mut self, altered_tiles: Vec<Option<AlteredTile>>) -> Self {
        self.altered_tiles = Some(altered_tiles);
        self
    }

    pub fn with_persisted(mut self, path: PathBuf) -> Self {
        self.persisted = Some(path);
        self
    }

    pub fn build(&self, w: WorldConfig, mod_: &Mod) -> Result<PathBuf, Error> {
        let (_, snapshot_path) = tempfile::NamedTempFile::new()?.keep()?;
        let tiles = self.tiles.tiles(&w, mod_);
        let individuals = self.individuals.individuals(&w, &tiles);
        let squads = self.squads.squads(&w, &individuals);
        let projectiles = self.projectiles.projectiles(&w, &tiles);
        let phase = self.phase;
        let duration = w.fight_duration();
        let altered_tiles = self
            .altered_tiles
            .clone()
            .unwrap_or_else(|| vec![None; tiles.len()]);

        let decor = vec![None; tiles.len()];
        let smoke = vec![oc_utils::percent::Percent::ZERO; tiles.len()];

        let mut snapshot = Snapshot {
            w,
            tiles,
            altered_tiles,
            craters: vec![],
            wrecks: vec![],
            decor,
            smoke,
            smoke_clouds: vec![],
            individuals,
            squads,
            vehicles: self.vehicles.clone(),
            projectiles,
            phase,
            duration,
            start: None,
            spawns: BySide::new(self.side_a_spawns.clone(), self.side_b_spawns.clone()),
            supports: self.supports.clone(),
            pending_supports: vec![],
            support_realized: vec![],
            uuids: Default::default(),
        };
        if let Some(path) = &self.persisted {
            let state = State::load(path)?;
            oc_world_server::generate::apply_persisted_map(&mut snapshot, state)
                .map_err(|e| Error::Persist(path.clone(), e))?;
        }
        snapshot.save(&snapshot_path)?;

        Ok(snapshot_path)
    }
}

impl<T, I, S> SnapshotBuilder<T, I, S, Vec<Projectile>>
where
    T: tile::TilesGenerator,
    I: individual::IndividualsGenerator,
    S: squad::SquadsGenerator,
{
    /// Fight phase snapshot, without projectiles nor spawn zones
    pub fn fight(tiles: T, individuals: I, squads: S) -> Self {
        Self::new(
            tiles,
            individuals,
            squads,
            vec![],
            BattlePhase::Fight,
            vec![],
            vec![],
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tempfile keep error: {0}")]
    Keep(#[from] tempfile::PersistError),
    #[error("Build error: {0}")]
    Save(#[from] FileError),
    #[error("Apply persisted map {0}: {1}")]
    Persist(PathBuf, anyhow::Error),
}

pub struct EmptyGenerator<T>(PhantomData<T>);

impl<T> Default for EmptyGenerator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EmptyGenerator<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl individual::IndividualsGenerator for EmptyGenerator<Individual> {
    fn individuals(&self, _: &WorldConfig, _: &[Tile]) -> Vec<Individual> {
        vec![]
    }
}

impl squad::SquadsGenerator for EmptyGenerator<Squad> {
    fn squads(&self, _: &WorldConfig, _: &[Individual]) -> Vec<Squad> {
        vec![]
    }
}

impl projectile::ProjectilesGenerator for EmptyGenerator<Projectile> {
    fn projectiles(&self, _: &WorldConfig, _: &[Tile]) -> Vec<Projectile> {
        vec![]
    }
}

impl individual::IndividualsGenerator for Vec<Individual> {
    fn individuals(&self, _: &WorldConfig, _: &[Tile]) -> Vec<Individual> {
        self.clone()
    }
}

impl projectile::ProjectilesGenerator for Vec<Projectile> {
    fn projectiles(&self, _: &WorldConfig, _: &[Tile]) -> Vec<Projectile> {
        self.clone()
    }
}

impl squad::SquadsGenerator for Vec<Squad> {
    fn squads(&self, _: &WorldConfig, _: &[Individual]) -> Vec<Squad> {
        self.clone()
    }
}

impl<T: Fn(&WorldConfig, &[Tile]) -> Vec<Individual>> individual::IndividualsGenerator for T {
    fn individuals(&self, w: &WorldConfig, tiles: &[Tile]) -> Vec<Individual> {
        self(w, tiles)
    }
}

impl<T: Fn(&WorldConfig, &[Individual]) -> Vec<Squad>> squad::SquadsGenerator for T {
    fn squads(&self, w: &WorldConfig, individuals: &[Individual]) -> Vec<Squad> {
        self(w, individuals)
    }
}

impl<T: Fn(&WorldConfig, &[Tile]) -> Vec<Projectile>> projectile::ProjectilesGenerator for T {
    fn projectiles(&self, w: &WorldConfig, tiles: &[Tile]) -> Vec<Projectile> {
        self(w, tiles)
    }
}
