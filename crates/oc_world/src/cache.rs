use std::path::PathBuf;

use image::DynamicImage;
use oc_geo::region::{RegionXy, WorldRegionIndex};
use oc_root::{WcfgInto, WorldConfig};
use thiserror::Error;

pub fn cache_region_background(
    w: &WorldConfig,
    output: &PathBuf,
    image: &DynamicImage,
    i: WorldRegionIndex,
) -> Result<(), CacheRegionBackgroundError> {
    let xy: RegionXy = i.into_(w);
    let x = xy.0.0 * w.region_width_pixels();
    let y = xy.0.1 * w.region_height_pixels();
    let width = w.region_width_pixels() as u32;
    let height = w.region_height_pixels() as u32;
    let region = image.crop_imm(x as u32, y as u32, width, height);
    region.save(output)?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum CacheRegionBackgroundError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::error::ImageError),
}
