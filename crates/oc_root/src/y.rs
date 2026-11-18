use glam::{Vec2, Vec3};

use crate::WorldConfig;
use crate::geo::WorldVec2;
use crate::geo::WorldVec3;

// TODO: T not necessary ? Use Self
pub trait Y<T> {
    fn to_world_y(&self, w: &WorldConfig) -> T;
    fn to_gui_y(&self, w: &WorldConfig) -> T;
}

impl Y<f32> for f32 {
    fn to_world_y(&self, w: &WorldConfig) -> f32 {
        w.world_height_pixels as f32 - self
    }

    fn to_gui_y(&self, w: &WorldConfig) -> f32 {
        w.world_height_pixels as f32 - self
    }
}

impl Y<Vec2> for Vec2 {
    fn to_world_y(&self, w: &WorldConfig) -> Vec2 {
        Vec2::new(self.x, w.world_height_pixels as f32 - self.y)
    }

    fn to_gui_y(&self, w: &WorldConfig) -> Vec2 {
        Vec2::new(self.x, w.world_height_pixels as f32 - self.y)
    }
}

impl Y<Vec3> for Vec3 {
    fn to_world_y(&self, w: &WorldConfig) -> Vec3 {
        Vec3::new(self.x, w.world_height_pixels as f32 - self.y, self.z)
    }

    fn to_gui_y(&self, w: &WorldConfig) -> Vec3 {
        Vec3::new(self.x, w.world_height_pixels as f32 - self.y, self.z)
    }
}

impl Y<[f32; 3]> for [f32; 3] {
    fn to_world_y(&self, w: &WorldConfig) -> [f32; 3] {
        [self[0], w.world_height_pixels as f32 - self[1], self[2]]
    }

    fn to_gui_y(&self, w: &WorldConfig) -> [f32; 3] {
        [self[0], w.world_height_pixels as f32 - self[1], self[2]]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum V {
    Server,
    Gui,
}

impl Y<WorldVec3> for WorldVec3 {
    fn to_world_y(&self, w: &WorldConfig) -> WorldVec3 {
        WorldVec3::new(self.x, w.world_height_pixels as f32 - self.y, self.z)
    }

    fn to_gui_y(&self, w: &WorldConfig) -> WorldVec3 {
        WorldVec3::new(self.x, w.world_height_pixels as f32 - self.y, self.z)
    }
}

impl Y<WorldVec2> for WorldVec2 {
    fn to_world_y(&self, w: &WorldConfig) -> WorldVec2 {
        WorldVec2::new(self.x, w.world_height_pixels as f32 - self.y)
    }

    fn to_gui_y(&self, w: &WorldConfig) -> WorldVec2 {
        WorldVec2::new(self.x, w.world_height_pixels as f32 - self.y)
    }
}
