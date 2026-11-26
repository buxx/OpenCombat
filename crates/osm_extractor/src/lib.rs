//! Extract OpenStreetMap data and elevation for a GPS area, and render map backgrounds.
//!
//! - `extract` bin: download roads, buildings, water, forest (GeoJSON) and height (16 bits PNG).
//! - `build_map` bin: render PNG backgrounds from an extraction directory with the default OSM theme.

pub mod feature;
pub mod geo;
pub mod height;
pub mod meta;
pub mod overpass;
pub mod render;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}
