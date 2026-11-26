use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::geo::BBox;

pub const META_FILE: &str = "meta.json";

/// Description of an extraction directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    /// Requested area.
    pub bbox: BBox,
    /// Approximate area size in meters (width, height).
    pub size_meters: (f64, f64),
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<HeightMeta>,
}

/// Elevation raster description. The PNG is 16 bits grayscale:
/// `elevation_m = min + value / 65535 * (max - min)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightMeta {
    pub file: String,
    pub width: u32,
    pub height: u32,
    /// Minimum elevation in meters (pixel value 0).
    pub min: f64,
    /// Maximum elevation in meters (pixel value 65535).
    pub max: f64,
    /// Source tiles zoom level.
    pub zoom: u8,
    /// Approximate ground resolution in meters per pixel.
    pub meters_per_pixel: f64,
    pub source: String,
}

impl Meta {
    pub fn load(dir: &Path) -> Result<Self> {
        let path = dir.join(META_FILE);
        let content =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let path = dir.join(META_FILE);
        std::fs::write(&path, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("write {}", path.display()))
    }
}
