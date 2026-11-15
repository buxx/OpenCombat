use std::ops::Deref;

use derive_more::{Add, Deref, Div, Mul, Sub};

use crate::WorldConfig;

#[oc_derive::archivable(serde)]
#[derive(Debug, PartialEq, Clone, Copy, Deref)]
pub struct Seconds(pub f32);

impl std::ops::Div<f32> for Seconds {
    type Output = Seconds;

    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl std::ops::Add<f32> for Seconds {
    type Output = Seconds;

    fn add(self, rhs: f32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl std::ops::Mul<f32> for Seconds {
    type Output = Seconds;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy, Default)]
pub struct Meters(pub f32);

impl Meters {
    pub const fn pixels(&self, w: &WorldConfig) -> f32 {
        self.0 * w.geo_pixels_per_meters
    }
}

impl std::ops::Mul<f32> for Meters {
    type Output = Meters;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, PartialEq, Clone, Copy, PartialOrd, Add, Sub, Mul, Div)]
pub struct MetersSeconds(pub f32);

impl Deref for MetersSeconds {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
