use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use oc_individual::IndividualIndex;
use oc_physics::Event;

use oc_root::geo::{WorldVec2, WorldVec3};

use crate::{runner::update::ExplosionOrigin, state::ObjectId};

#[derive(Debug, Clone, Default)]
pub struct Tracker {
    inner: Arc<Mutex<Tracker_>>,
}

impl Tracker {
    pub fn take(&self) -> std::sync::MutexGuard<'_, Tracker_> {
        self.inner
            .lock()
            .expect("Assume tracker is always available")
    }
}

#[derive(Debug, Default)]
pub struct Tracker_ {
    pub physics: Vec<Event<ObjectId>>,
    pub individuals: Vec<(IndividualIndex, oc_individual::Update)>,
    pub individual_positions: Vec<(IndividualIndex, WorldVec3)>,
    pub explosions: Vec<(WorldVec2, ExplosionOrigin)>,
    /// Individuals status changes, with the (real) instant they were applied
    pub statuses: Vec<(Instant, IndividualIndex, oc_individual::Status)>,
}

impl Tracker_ {
    pub fn push_individual(&mut self, i: IndividualIndex, update: &oc_individual::Update) {
        if let oc_individual::Update::SetStatus(status) = update {
            self.statuses.push((Instant::now(), i, *status));
        }
        self.individuals.push((i, update.clone()));
    }
}
