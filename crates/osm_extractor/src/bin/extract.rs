//! Extract OSM data and elevation for an area.
//!
//! Example (Sainte-Mère-Église):
//! `cargo run -p osm_extractor --bin extract -- --top-left 49.4130,-1.3240 --top-right 49.4130,-1.3080 --bottom-left 49.4030,-1.3240 --bottom-right 49.4030,-1.3080 --output maps_src/sme`

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;
use osm_extractor::{
    feature::{DataType, FeatureCollection},
    geo::{Corners, LatLon},
    height,
    meta::Meta,
    overpass::{DEFAULT_OVERPASS_URL, Overpass},
};

/// Max area side, in meters (protect public Overpass instance).
const MAX_SIDE_M: f64 = 20_000.0;

#[derive(Parser, Debug)]
#[command(about = "Extract OSM data (road/building/water/forest) and height for a GPS area")]
struct Args {
    /// Top left (north west) corner, as "lat,lon" (ex: 49.4130,-1.3240)
    #[arg(long, allow_hyphen_values = true)]
    top_left: LatLon,
    /// Top right (north east) corner, as "lat,lon"
    #[arg(long, allow_hyphen_values = true)]
    top_right: LatLon,
    /// Bottom left (south west) corner, as "lat,lon"
    #[arg(long, allow_hyphen_values = true)]
    bottom_left: LatLon,
    /// Bottom right (south east) corner, as "lat,lon"
    #[arg(long, allow_hyphen_values = true)]
    bottom_right: LatLon,
    /// Output directory
    #[arg(long, short)]
    output: PathBuf,
    /// Data types to extract
    #[arg(
        long,
        short,
        value_delimiter = ',',
        default_value = "road,building,water,forest,height"
    )]
    types: Vec<DataType>,
    /// Elevation tiles zoom level (15 = ~4.8m/px at equator, 14 = ~9.5m/px)
    #[arg(long, default_value_t = 15)]
    height_zoom: u8,
    /// Elevation tiles URL template (Terrarium encoding)
    #[arg(long, default_value = height::DEFAULT_TILES_URL)]
    height_url: String,
    /// Overpass API endpoint
    #[arg(long, default_value = DEFAULT_OVERPASS_URL)]
    overpass_url: String,
}

fn main() -> Result<()> {
    osm_extractor::init_tracing();
    let args = Args::parse();

    let corners = Corners {
        top_left: args.top_left,
        top_right: args.top_right,
        bottom_left: args.bottom_left,
        bottom_right: args.bottom_right,
    };
    if let Err(error) = corners.validate() {
        bail!("invalid corners: {error}");
    }
    let bbox = corners.bbox();
    let deviation = corners.max_deviation_m();
    if deviation > 1.0 {
        tracing::info!(
            "Corners are not a perfect rectangle (up to {deviation:.0}m off): \
             using min/max latitude and longitude"
        );
    }
    let size = bbox.size_meters();
    if size.0 > MAX_SIDE_M || size.1 > MAX_SIDE_M {
        bail!(
            "area too large ({:.0}m x {:.0}m), max side is {MAX_SIDE_M}m",
            size.0,
            size.1
        );
    }
    tracing::info!("Area {bbox} ({:.0}m x {:.0}m)", size.0, size.1);

    std::fs::create_dir_all(&args.output)?;
    // Keep previous height info if same area and height not re-extracted
    let previous = Meta::load(&args.output).ok().filter(|m| m.bbox == bbox);
    let mut meta = Meta {
        bbox,
        size_meters: size,
        height: previous.and_then(|m| m.height),
    };

    let overpass = Overpass::new(&args.overpass_url)?;
    let mut types = args.types.clone();
    types.sort();
    types.dedup();
    for type_ in types {
        match type_ {
            DataType::Height => {
                let grid = height::fetch(&bbox, args.height_zoom, &args.height_url)?;
                let height_meta = height::save(
                    &grid,
                    &args.output,
                    &bbox,
                    args.height_zoom,
                    &args.height_url,
                )?;
                tracing::info!(
                    "height: {}x{} px, {:.1}m..{:.1}m -> {}",
                    height_meta.width,
                    height_meta.height,
                    height_meta.min,
                    height_meta.max,
                    args.output.join(&height_meta.file).display()
                );
                meta.height = Some(height_meta);
            }
            type_ => {
                tracing::info!("Fetch {type_} ...");
                let features = overpass.fetch(type_, &bbox)?;
                let path = args.output.join(type_.geojson_file());
                tracing::info!(
                    "{type_}: {} feature(s) -> {}",
                    features.len(),
                    path.display()
                );
                FeatureCollection::new(features).save(&path)?;
            }
        }
    }

    meta.save(&args.output)?;
    Ok(())
}
