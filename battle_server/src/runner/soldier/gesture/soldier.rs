use battle_core::{config::TARGET_FPS, entity::soldier::Soldier, game::weapon::Weapon};
use rand::Rng;

use crate::runner::soldier::SoldierRunner;

impl SoldierRunner {
    pub fn soldier_reloading_end(&self, _soldier: &Soldier, _weapon: &Weapon) -> u64 {
        // TODO : Depending multiple factor
        let mut rng = rand::thread_rng();
        self.battle_state().frame_i() + TARGET_FPS + rng.gen_range(0..50)
    }

    pub fn soldier_aiming_end(&self, _soldier: &Soldier, _weapon: &Weapon) -> u64 {
        // TODO : Depending multiple factor
        let mut rng = rand::thread_rng();
        self.battle_state().frame_i() + TARGET_FPS + rng.gen_range(0..50)
    }

    pub fn soldier_firing_end(&self, _soldier: &Soldier, _weapon: &Weapon) -> u64 {
        // TODO : Depending multiple factor like weapon, riffle or single shot etc
        let mut rng = rand::thread_rng();
        self.battle_state().frame_i() + 5 + rng.gen_range(0..50)
    }
}
