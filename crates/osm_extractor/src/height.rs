//! Elevation extraction from AWS "Terrarium" terrain tiles (free, no API key).
//! <https://registry.opendata.aws/terrain-tiles/>
//! Elevation encoding: `meters = (R * 256 + G + B / 256) - 32768`.

use std::{path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use image::{ImageBuffer, Luma};

use crate::{
    geo::{BBox, EARTH_RADIUS_M, tile_xy},
    meta::HeightMeta,
    overpass::USER_AGENT,
};

pub const DEFAULT_TILES_URL: &str =
    "https://s3.amazonaws.com/elevation-tiles-prod/terrarium/{z}/{x}/{y}.png";
pub const MAX_ZOOM: u8 = 15;
const TILE_SIZE: u32 = 256;
const MAX_TILES: u64 = 400;

pub const HEIGHT_FILE: &str = "height.png";

/// Elevation grid in meters, row major (north to south).
pub struct HeightGrid {
    pub width: u32,
    pub height: u32,
    pub values: Vec<f32>,
}

/// Download tiles covering the bbox and crop them to the bbox.
pub fn fetch(bbox: &BBox, zoom: u8, url_template: &str) -> Result<HeightGrid> {
    if zoom > MAX_ZOOM {
        bail!("height zoom must be <= {MAX_ZOOM}");
    }
    let (x0, y0) = tile_xy(bbox.north, bbox.west, zoom);
    let (x1, y1) = tile_xy(bbox.south, bbox.east, zoom);
    let (tx0, ty0) = (x0.floor() as u32, y0.floor() as u32);
    let (tx1, ty1) = (x1.floor() as u32, y1.floor() as u32);
    let tiles_count = u64::from(tx1 - tx0 + 1) * u64::from(ty1 - ty0 + 1);
    if tiles_count > MAX_TILES {
        bail!(
            "{tiles_count} elevation tiles required (max {MAX_TILES}): reduce area or --height-zoom"
        );
    }
    tracing::info!("Download {tiles_count} elevation tile(s) at zoom {zoom}");

    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(60))
        .build()?;

    let mosaic_width = (tx1 - tx0 + 1) * TILE_SIZE;
    let mosaic_height = (ty1 - ty0 + 1) * TILE_SIZE;
    let mut mosaic = vec![0f32; (mosaic_width * mosaic_height) as usize];

    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            let url = url_template
                .replace("{z}", &zoom.to_string())
                .replace("{x}", &tx.to_string())
                .replace("{y}", &ty.to_string());
            tracing::debug!("GET {url}");
            let bytes = client
                .get(&url)
                .send()
                .and_then(|r| r.error_for_status())
                .and_then(|r| r.bytes())
                .with_context(|| format!("download {url}"))?;
            let tile = image::load_from_memory(&bytes)
                .with_context(|| format!("decode {url}"))?
                .to_rgb8();
            let ox = (tx - tx0) * TILE_SIZE;
            let oy = (ty - ty0) * TILE_SIZE;
            for (px, py, rgb) in tile.enumerate_pixels() {
                let [r, g, b] = rgb.0;
                let elevation =
                    f32::from(r) * 256.0 + f32::from(g) + f32::from(b) / 256.0 - 32768.0;
                let index = (oy + py) * mosaic_width + ox + px;
                mosaic[index as usize] = elevation;
            }
        }
    }

    // Crop to bbox (pixel coordinates in mosaic)
    let tile_size = f64::from(TILE_SIZE);
    let cx0 = ((x0 - f64::from(tx0)) * tile_size).floor() as u32;
    let cy0 = ((y0 - f64::from(ty0)) * tile_size).floor() as u32;
    let cx1 = (((x1 - f64::from(tx0)) * tile_size).ceil() as u32).min(mosaic_width);
    let cy1 = (((y1 - f64::from(ty0)) * tile_size).ceil() as u32).min(mosaic_height);
    let width = (cx1 - cx0).max(1);
    let height = (cy1 - cy0).max(1);
    let mut values = Vec::with_capacity((width * height) as usize);
    for y in cy0..cy0 + height {
        let start = (y * mosaic_width + cx0) as usize;
        values.extend_from_slice(&mosaic[start..start + width as usize]);
    }

    Ok(HeightGrid {
        width,
        height,
        values,
    })
}

/// Approximate ground resolution of tiles at given zoom and latitude.
pub fn meters_per_pixel(zoom: u8, lat: f64) -> f64 {
    let circumference = 2.0 * std::f64::consts::PI * EARTH_RADIUS_M;
    circumference * lat.to_radians().cos() / (f64::from(TILE_SIZE) * f64::from(1u32 << zoom))
}

/// Save grid as normalized 16 bits grayscale PNG and return its description.
pub fn save(
    grid: &HeightGrid,
    dir: &Path,
    bbox: &BBox,
    zoom: u8,
    source: &str,
) -> Result<HeightMeta> {
    let min = grid.values.iter().copied().fold(f32::INFINITY, f32::min);
    let max = grid
        .values
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, f32::max);
    let range = (max - min).max(f32::EPSILON);
    let pixels: Vec<u16> = grid
        .values
        .iter()
        .map(|v| (((v - min) / range) * 65535.0).round() as u16)
        .collect();
    let image: ImageBuffer<Luma<u16>, Vec<u16>> =
        ImageBuffer::from_raw(grid.width, grid.height, pixels).context("build height image")?;
    let path = dir.join(HEIGHT_FILE);
    image
        .save(&path)
        .with_context(|| format!("save {}", path.display()))?;

    Ok(HeightMeta {
        file: HEIGHT_FILE.to_string(),
        width: grid.width,
        height: grid.height,
        min: f64::from(min),
        max: f64::from(max),
        zoom,
        meters_per_pixel: meters_per_pixel(zoom, (bbox.north + bbox.south) / 2.0),
        source: source.to_string(),
    })
}
