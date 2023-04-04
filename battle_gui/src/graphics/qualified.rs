use glam::Vec2;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Zoom {
    In,
    Standard,
    Out,
}

impl Zoom {
    pub fn default() -> Self {
        Self::Standard
    }

    pub fn factor(&self) -> f32 {
        match self {
            Zoom::In => 3.0,
            Zoom::Standard => 1.0,
            Zoom::Out => -5.0,
        }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.factor(), self.factor())
    }

    pub fn next(&self) -> Zoom {
        match self {
            Zoom::In => Zoom::In,
            Zoom::Standard => Zoom::In,
            Zoom::Out => Zoom::Standard,
        }
    }

    pub fn previous(&self) -> Zoom {
        match self {
            Zoom::In => Zoom::Standard,
            Zoom::Standard => Zoom::Out,
            Zoom::Out => Zoom::Out,
        }
    }
}
