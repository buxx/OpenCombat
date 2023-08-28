use super::{message::RunnerMessage, Runner};

mod bullet;
mod explosion;

impl Runner {
    pub fn tick_physics(&mut self) -> Vec<RunnerMessage> {
        puffin::profile_scope!("tick_physics");
        let mut messages = vec![];

        if self.battle_state.frame_i() % self.config.physics_update_freq() == 0 {
            messages.extend(self.tick_bullet_fires());
            messages.extend(self.tick_explosions());
        }

        messages
    }
}
