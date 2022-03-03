pub struct Config {
    target_fps: u32,
    entity_tick_freq: u64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            target_fps: 60,
            entity_tick_freq: 20,
        }
    }

    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }

    pub fn entity_tick_freq(&self) -> u64 {
        self.entity_tick_freq
    }
}
