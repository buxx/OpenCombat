use oc_root::format::RkyvFile;
use oc_vehicle::VehicleIndex;
use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Instant,
};

#[cfg(feature = "perfs")]
use crate::perf::Perf;
use oc_geo::tile::WorldTileIndex;
use oc_individual::IndividualIndex;
use oc_mod::Mod;
use oc_projectile::ProjectileId;
use oc_root::{Client, WorldConfig, ids::Ids};
use oc_world::{
    World, load::WorldLoader, reader::MapReader, snapshot::Snapshot, wreck::WreckIndex,
};
use serde::{Deserialize, Serialize};

use crate::{
    config::ServerConfig, index::Indexes, outline, routing::Listeners, runner::update::Update,
    rw_lock, rw_lock_imports,
};

rw_lock_imports!();

#[derive(Clone)]
pub struct State<E: Client> {
    pub w: WorldConfig,
    pub _ids: Ids,
    pub _mod: Mod,
    #[cfg(feature = "perfs")]
    pub perf: Arc<Perf>,
    pub world: Arc<RwLock<World>>,
    pub indexes: Arc<RwLock<Indexes>>,
    pub listeners: Arc<RwLock<Listeners<E>>>,
    pub scheduled: Arc<Mutex<Vec<(Instant, Update)>>>,
}

impl<E: Client> State<E> {
    pub fn new(w: WorldConfig, ids: Ids, mod_: Mod, world: World) -> Self {
        #[cfg(feature = "perfs")]
        let perf = Arc::new(Perf::default());
        let indexes = Arc::new(rw_lock!(Indexes::new(&world, &w)));
        let world = Arc::new(rw_lock!(world));
        let listeners = Arc::new(rw_lock!(Listeners::new(&w)));
        let scheduled = Arc::new(Mutex::new(vec![]));

        Self {
            w,
            _ids: ids,
            _mod: mod_,
            #[cfg(feature = "perfs")]
            perf,
            world,
            indexes,
            listeners,
            scheduled,
        }
    }

    pub fn _mod(&self) -> &Mod {
        &self._mod
    }

    pub fn world(&self) -> RwLockReadGuard<'_, World> {
        self.world.read().expect("Assume lock")
    }

    pub fn world_mut(&self) -> RwLockWriteGuard<'_, World> {
        self.world.write().expect("Assume lock")
    }

    pub fn indexes(&self) -> RwLockReadGuard<'_, Indexes> {
        self.indexes.read().expect("Assume lock")
    }

    pub fn indexes_mut(&self) -> RwLockWriteGuard<'_, Indexes> {
        self.indexes.write().expect("Assume lock")
    }

    pub fn listeners(&self) -> RwLockReadGuard<'_, Listeners<E>> {
        self.listeners.read().expect("Assume lock")
    }

    pub fn listeners_mut(&self) -> RwLockWriteGuard<'_, Listeners<E>> {
        self.listeners.write().expect("Assume lock")
    }

    pub fn scheduled(&self) -> MutexGuard<'_, Vec<(Instant, Update)>> {
        self.scheduled.lock().expect("Assume lock")
    }
}

// TODO: move code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectId {
    Individual(IndividualIndex),
    Projectile(ProjectileId),
    Tile(WorldTileIndex),
    Vehicle(VehicleIndex),
    Wreck(WreckIndex),
}

pub fn init<E: Client>(config: ServerConfig, map: &MapReader) -> Result<State<E>, anyhow::Error> {
    let cache = config.roots.clone();
    let world = config.world.clone();
    let mod_ = config.mod_.clone();

    let ids = Ids::default();
    let mod_path = mod_.clone();
    let mod_ = Mod::load(&mod_, Some(&cache))?;
    outline::cache_soldiers_outline(&mod_, &mod_path, &cache)?;
    let snapshot = Snapshot::load(&config.snapshot)?;
    let w = snapshot
        .w
        .clone()
        .with_region_width(config.region_width)
        .with_region_height(config.region_height);
    let world = WorldLoader::new(w.clone(), mod_.clone(), world.clone(), cache.clone());
    let world = world.load(&ids, snapshot, map)?;

    Ok(State::new(w.clone(), ids, mod_.clone(), world))
}
