#[cfg(feature = "bevy")]
use bevy::math::Vec2 as BevyVec2;
#[cfg(feature = "bevy")]
use bevy::math::Vec3 as BevyVec3;
use derive_more::{Constructor, From, Into};
use glam::Vec2;
use glam::Vec3;

use crate::{WcfgFrom, y::Y};

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Constructor, From, Into)]
pub struct WorldVec2 {
    pub x: f32,
    pub y: f32,
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Constructor, From, Into)]
pub struct WorldVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Constructor, From, Into)]
pub struct ScreenVec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Constructor, From, Into)]
pub struct ScreenVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

macro_rules! impl_array_from {
    ($ty:ty, $n:literal, [$($field:ident),+]) => {
        impl From<[f32; $n]> for $ty {
            fn from([$($field),+]: [f32; $n]) -> Self {
                Self { $($field),+ }
            }
        }

        impl From<$ty> for [f32; $n] {
            fn from(p: $ty) -> Self {
                [$(p.$field),+]
            }
        }
    };
}

impl_array_from!(WorldVec2, 2, [x, y]);
impl_array_from!(WorldVec3, 3, [x, y, z]);
impl_array_from!(ScreenVec2, 2, [x, y]);

impl WorldVec2 {
    pub fn extend(&self, z: f32) -> WorldVec3 {
        WorldVec3::new(self.x, self.y, z)
    }

    #[inline]
    pub fn normalize_or_zero(&self) -> Self {
        let vec = Vec2::new(self.x, self.y).normalize_or_zero();
        Self::new(vec.x, vec.y)
    }
}

impl std::ops::Sub for WorldVec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::SubAssign for WorldVec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl From<WorldVec3> for WorldVec2 {
    fn from(value: WorldVec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<WorldVec3> for Vec3 {
    fn from(value: WorldVec3) -> Vec3 {
        Vec3::new(value.x, value.y, value.z)
    }
}

impl From<WorldVec3> for Vec2 {
    fn from(value: WorldVec3) -> Vec2 {
        Vec2::new(value.x, value.y)
    }
}

impl WcfgFrom<ScreenVec2> for WorldVec2 {
    fn from_(value: ScreenVec2, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_world_y(w),
        }
    }
}

#[cfg(feature = "bevy")]
impl WcfgFrom<BevyVec2> for WorldVec2 {
    fn from_(value: BevyVec2, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_world_y(w),
        }
    }
}

impl From<WorldVec2> for Vec2 {
    fn from(value: WorldVec2) -> Self {
        Vec2::new(value.x, value.y)
    }
}

impl WorldVec3 {
    #[inline]
    pub fn normalize_or_zero(&self) -> Self {
        let vec = Vec3::new(self.x, self.y, self.z).normalize_or_zero();
        Self::new(vec.x, vec.y, vec.z)
    }

    pub fn distance(&self, other: WorldVec3) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;

        // Calculate the absolute distance
        ((dx * dx) + (dy * dy) + (dz * dz)).sqrt()
    }

    pub fn to_vec2(&self) -> WorldVec2 {
        WorldVec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn with_z(&self, z: f32) -> WorldVec3 {
        WorldVec3 {
            x: self.x,
            y: self.y,
            z,
        }
    }

    pub fn add_z(&self, z: f32) -> WorldVec3 {
        WorldVec3 {
            x: self.x,
            y: self.y,
            z: self.z + z,
        }
    }
}

impl std::ops::Sub for WorldVec3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::SubAssign for WorldVec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl ScreenVec2 {
    pub fn extend(&self, z: f32) -> ScreenVec3 {
        ScreenVec3::new(self.x, self.y, z)
    }
}

impl WcfgFrom<WorldVec2> for ScreenVec2 {
    fn from_(value: WorldVec2, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_gui_y(w),
        }
    }
}

impl WcfgFrom<WorldVec3> for ScreenVec2 {
    fn from_(value: WorldVec3, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_gui_y(w),
        }
    }
}

#[cfg(feature = "bevy")]
impl From<BevyVec2> for ScreenVec2 {
    fn from(value: BevyVec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[cfg(feature = "bevy")]
impl From<ScreenVec2> for BevyVec2 {
    fn from(val: ScreenVec2) -> Self {
        BevyVec2::new(val.x, val.y)
    }
}

impl WcfgFrom<WorldVec3> for ScreenVec3 {
    fn from_(value: WorldVec3, w: &crate::WorldConfig) -> Self {
        Self {
            x: value.x,
            y: value.y.to_gui_y(w),
            z: value.z,
        }
    }
}

#[cfg(feature = "bevy")]
impl From<ScreenVec3> for BevyVec3 {
    fn from(val: ScreenVec3) -> Self {
        BevyVec3::new(val.x, val.y, val.z)
    }
}
