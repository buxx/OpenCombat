use battle_core::{config::TARGET_FPS, entity::soldier::Soldier, game::weapon::Weapon};
use rand::Rng;

use crate::runner::Runner;

impl Runner {
    pub fn soldier_reloading_end(&self, _soldier: &Soldier, weapon: &Weapon) -> u64 {
        // TODO : Depending multiple factor
        let mut rng = rand::thread_rng();
        self.battle_state.frame_i() + TARGET_FPS + weapon.reloading_frames() + rng.gen_range(0..50)
    }

    pub fn soldier_aiming_end(&self, _soldier: &Soldier, weapon: &Weapon) -> u64 {
        // TODO : Depending multiple factor
        let mut rng = rand::thread_rng();
        self.battle_state.frame_i() + TARGET_FPS + weapon.aiming_frames() + rng.gen_range(0..50)
    }

    pub fn soldier_firing_end(&self, _soldier: &Soldier, weapon: &Weapon) -> u64 {
        // TODO : Depending multiple factor like weapon, riffle or single shot etc
        let mut rng = rand::thread_rng();
        // FIXME: firing_frames depend on Shot type
        self.battle_state.frame_i() + 5 + weapon.firing_frames() + rng.gen_range(0..50)
    }
}
