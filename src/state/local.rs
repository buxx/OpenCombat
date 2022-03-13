use glam::Vec2;

pub struct LocalState {
    /// Printed frames since start of program
    pub frame_i: u64,
    /// Offset to apply to battle scene by window relative
    pub display_scene_offset: Vec2,
    /// Scale to apply to battle scene by window relative
    pub display_scene_scale: Vec2,
    // Bellow, some player display configurations
    pub draw_decor: bool,
}

impl LocalState {
    pub fn new() -> Self {
        Self {
            frame_i: 0,
            display_scene_offset: Vec2::new(0., 0.),
            display_scene_scale: Vec2::new(1., 1.),
            draw_decor: true,
        }
    }
}
