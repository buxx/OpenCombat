use glam::Vec2;
use serde::{Deserialize, Serialize};

#[cfg(feature = "hd")]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Zoom {
    In,
    Standard,
}

#[cfg(not(feature = "hd"))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Zoom {
    Standard,
}

impl Zoom {
    pub fn default() -> Self {
        Self::Standard
    }

    pub fn hd() -> Self {
        #[cfg(feature = "hd")]
        {
            Self::In
        }
        #[cfg(not(feature = "hd"))]
        {
            Self::Standard
        }
    }

    pub fn factor(&self) -> f32 {
        #[cfg(feature = "hd")]
        {
            match self {
                Zoom::In => 3.0,
                Zoom::Standard => 1.0,
            }
        }
        #[cfg(not(feature = "hd"))]
        {
            match self {
                Zoom::Standard => 1.0,
            }
        }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.factor(), self.factor())
    }

    pub fn next(&self) -> Zoom {
        #[cfg(feature = "hd")]
        {
            match self {
                Zoom::In => Zoom::In,
                Zoom::Standard => Zoom::In,
            }
        }
        #[cfg(not(feature = "hd"))]
        {
            match self {
                Zoom::Standard => Zoom::Standard,
            }
        }
    }

    pub fn previous(&self) -> Zoom {
        #[cfg(feature = "hd")]
        {
            match self {
                Zoom::In => Zoom::Standard,
                Zoom::Standard => Zoom::Standard,
            }
        }
        #[cfg(not(feature = "hd"))]
        {
            match self {
                Zoom::Standard => Zoom::Standard,
            }
        }
    }

    pub fn suffix(&self) -> &str {
        #[cfg(feature = "hd")]
        {
            match self {
                Zoom::In => "__HD",
                _ => "",
            }
        }
        #[cfg(not(feature = "hd"))]
        {
            ""
        }
    }

    pub fn is_hd(&self) -> bool {
        #[cfg(feature = "hd")]
        {
            match self {
                Zoom::In => true,
                _ => false,
            }
        }
        #[cfg(not(feature = "hd"))]
        {
            false
        }
    }
}
