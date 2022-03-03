use rayon::prelude::*;

use crate::{entity::Entity, message::Message, types::*};

mod react;
mod tick;

pub struct Orchestrator {
    entities: Vec<Box<dyn Entity + Send + Sync>>,
}

impl Orchestrator {
    pub fn new(entities: Vec<Box<dyn Entity + Send + Sync>>) -> Orchestrator {
        Orchestrator { entities }
    }

    pub fn operate(&mut self) {
        loop {
            let results: Vec<Message> = (0..self.entities.len())
                .into_par_iter()
                .flat_map(|i| self.tick_entity(EntityIndex::from(i)))
                .collect();
            self.react(results);
        }
    }
}
