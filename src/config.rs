pub struct Config {
    target_fps: u32,
    entity_update_freq: u64,
    entity_animate_freq: u64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            /// Target FPS of engine
            target_fps: 60,
            /// Frequency of entity update :
            ///  - World pixel position according to movement
            ///  - ...
            entity_update_freq: 1,
            /// Frequency of entity animation :
            ///  - Compute visibility with other entities
            ///  - Compute behavior against physics (explosions, gunfires, ...)
            entity_animate_freq: 20,
        }
    }

    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }

    pub fn entity_update_freq(&self) -> u64 {
        self.entity_update_freq
    }

    pub fn entity_animate_freq(&self) -> u64 {
        self.entity_animate_freq
    }
}
