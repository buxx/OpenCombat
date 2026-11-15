use oc_geo::region::WorldRegionIndex;
use oc_root::{WorldConfig, identity::Identity, side::Side};
use oc_utils::index::AtIndex;
use rustc_hash::{FxHashMap, FxHashSet};
use std::hash::Hash;

pub struct Listeners<T: Clone + PartialEq + Hash + std::cmp::Eq> {
    // All endpoints currently listening something (aka all clients)
    all: Vec<T>,
    identities: FxHashMap<T, Identity>,
    // Endpoints listening side
    side_identitides: FxHashMap<Side, Vec<T>>,
    // Endpoints listening specifics regions (level 1 vector is all regions vector)
    regions_listeners: Vec<Vec<T>>,
    // Which regions listen
    listeners_regions: FxHashMap<T, Vec<WorldRegionIndex>>,
}

impl<T: Clone + PartialEq + Hash + std::cmp::Eq> Listeners<T> {
    pub fn new(w: &WorldConfig) -> Self {
        Self {
            all: vec![],
            side_identitides: FxHashMap::default(),
            identities: FxHashMap::default(),
            regions_listeners: vec![vec![]; w.regions_count() as usize],
            listeners_regions: FxHashMap::default(),
        }
    }

    pub fn push(&mut self, endpoint: T) {
        self.all.push(endpoint)
    }

    pub fn identify(&mut self, listener: T, identity: Identity) {
        self.identities.insert(listener.clone(), identity.clone());
        self.side_identitides
            .entry(identity.side)
            .or_default()
            .push(listener);
    }

    pub fn identity(&self, listener: &T) -> Option<&Identity> {
        self.identities.get(listener)
    }

    // TODO: test me
    pub fn remove(&mut self, listener: &T) {
        self.all.retain(|l| l != listener);
        if let Some(identity) = self.identities.remove(listener)
            && let Some(identities) = self.side_identitides.get_mut(&identity.side)
        {
            identities.retain(|l| l != listener);
        }

        let regions = self.listener_regions(listener).clone();
        for region in regions {
            self.forgot_region(listener, region)
        }
    }

    pub fn listen_region(&mut self, listener: &T, region: WorldRegionIndex) {
        tracing::trace!(name = "listeners-listen-region", region = ?region);
        self.regions_listeners.at_mut(region).push(listener.clone());
        self.listeners_regions
            .entry(listener.clone())
            .or_default()
            .push(region);
    }

    pub fn forgot_region(&mut self, listener: &T, region: WorldRegionIndex) {
        self.regions_listeners
            .at_mut(region)
            .retain(|l| l != listener);
    }

    // TODO: maybe more performant to return references ?
    pub fn find(&self, filter: Listening) -> FxHashSet<T> {
        match filter {
            Listening::Regions(regions) => regions
                .into_iter()
                .map(|tile| {
                    let region: WorldRegionIndex = tile;
                    self.regions_listeners.at(region).clone()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .flatten()
                .collect::<FxHashSet<_>>(),
            Listening::EnterBorder(before, after) => {
                let before_listeners = self.find(Listening::Regions(vec![before]));
                let after_listeners = self.find(Listening::Regions(vec![after]));
                after_listeners
                    .into_iter()
                    .filter(|l| !before_listeners.contains(l))
                    .collect::<FxHashSet<_>>()
            }
            Listening::ExitBorder(before, after) => {
                let before_listeners = self.find(Listening::Regions(vec![before]));
                let after_listeners = self.find(Listening::Regions(vec![after]));
                before_listeners
                    .into_iter()
                    .filter(|l| !after_listeners.contains(l))
                    .collect::<FxHashSet<_>>()
            }
            Listening::Side(side) => match self.side_identitides.get(&side) {
                Some(listeners) => listeners.clone().into_iter().collect::<FxHashSet<_>>(),
                None => FxHashSet::default(),
            },
            Listening::Any => self.all.iter().cloned().collect::<FxHashSet<_>>(),
        }
    }

    pub fn listener_regions(&self, listener: &T) -> &Vec<WorldRegionIndex> {
        static EMPTY: Vec<WorldRegionIndex> = vec![];
        self.listeners_regions.get(listener).unwrap_or(&EMPTY)
    }
}

#[derive(Debug, Clone)]
pub enum Listening {
    Any,
    /// Will match with all listener of one of these regions
    Regions(Vec<WorldRegionIndex>),
    /// Will match with all listener NOT listening region1 and listening region2
    EnterBorder(WorldRegionIndex, WorldRegionIndex),
    /// Will match with all listener listening region1 and NOT listening region2
    ExitBorder(WorldRegionIndex, WorldRegionIndex),
    /// Will match with all listener which are this side
    Side(Side),
}

#[cfg(feature = "debug")]
impl Listening {
    pub fn match_side(&self, side: Side) -> bool {
        match self {
            Listening::Side(side_) => *side_ == side,
            Listening::Any
            | Listening::Regions(_)
            | Listening::EnterBorder(_, _)
            | Listening::ExitBorder(_, _) => true,
        }
    }
}
