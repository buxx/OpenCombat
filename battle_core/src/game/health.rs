use oc_core::health::Health;

use crate::entity::soldier::Soldier;

pub struct SoldierHealthBuilder<'a> {
    soldier: &'a Soldier,
}

impl<'a> SoldierHealthBuilder<'a> {
    pub fn new(soldier: &'a Soldier) -> Self {
        Self { soldier }
    }

    pub fn build(&self) -> Health {
        if !self.soldier.alive() {
            return Health::Dead;
        }

        if self.soldier.unconscious() {
            return Health::Unconscious;
        }

        Health::Good
    }
}
