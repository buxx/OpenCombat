//! Render map backgrounds (PNG) from an `extract` output directory, with default OSM theme colors.
//!
//! Produces one transparent PNG per layer (`road.png`, `building.png`, `water.png`, `forest.png`)
//! and `map.png` with all layers composed on the OSM land color.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use osm_extractor::{
    feature::{DataType, FeatureCollection},
    geo::Projector,
    meta::Meta,
    render::{Renderer, colors},
};

const MAX_IMAGE_SIDE: f64 = 16_384.0;

#[derive(Parser, Debug)]
#[command(about = "Render PNG map backgrounds from extracted OSM data")]
struct Args {
    /// Directory produced by `extract`
    #[arg(long, short)]
    input: PathBuf,
    /// Output directory (default: input directory)
    #[arg(long, short)]
    output: Option<PathBuf>,
    /// Ground resolution, in pixels per meter
    #[arg(long, default_value_t = 1.0)]
    pixels_per_meter: f64,
    /// Layers to render
    #[arg(
        long,
        short,
        value_delimiter = ',',
        default_value = "road,building,water,forest"
    )]
    layers: Vec<DataType>,
    /// Do not produce the composed `map.png`
    #[arg(long)]
    no_composite: bool,
}

fn main() -> Result<()> {
    osm_extractor::init_tracing();
    let args = Args::parse();
    let output = args.output.clone().unwrap_or_else(|| args.input.clone());
    std::fs::create_dir_all(&output)?;

    if args.pixels_per_meter <= 0.0 {
        bail!("--pixels-per-meter must be > 0");
    }
    let meta = Meta::load(&args.input)?;
    let (width_m, height_m) = meta.size_meters;
    let width = (width_m * args.pixels_per_meter).round();
    let height = (height_m * args.pixels_per_meter).round();
    if width > MAX_IMAGE_SIDE || height > MAX_IMAGE_SIDE {
        bail!(
            "image would be {width}x{height} px (max side {MAX_IMAGE_SIDE}): decrease --pixels-per-meter"
        );
    }
    let (width, height) = ((width as u32).max(1), (height as u32).max(1));
    tracing::info!(
        "Render {width}x{height} px ({} px/m)",
        args.pixels_per_meter
    );

    let renderer = Renderer::new(
        Projector::new(meta.bbox, width, height),
        args.pixels_per_meter,
    );
    let mut composite = (!args.no_composite).then(|| renderer.new_pixmap(Some(colors::LAND)));

    // Draw in layering order (forest, water, building, road)
    for type_ in DataType::LAYERS.iter().filter(|t| args.layers.contains(t)) {
        let source = args.input.join(type_.geojson_file());
        if !source.exists() {
            tracing::warn!("{} not found, skip {type_}", source.display());
            continue;
        }
        let collection = FeatureCollection::load(&source)?;

        let mut pixmap = renderer.new_pixmap(None);
        renderer.draw(&mut pixmap, *type_, &collection);
        let path = output.join(format!("{}.png", type_.name()));
        pixmap
            .save_png(&path)
            .with_context(|| format!("save {}", path.display()))?;
        tracing::info!(
            "{type_}: {} feature(s) -> {}",
            collection.features.len(),
            path.display()
        );

        if let Some(composite) = &mut composite {
            renderer.draw(composite, *type_, &collection);
        }
    }

    if let Some(composite) = composite {
        let path = output.join("map.png");
        composite
            .save_png(&path)
            .with_context(|| format!("save {}", path.display()))?;
        tracing::info!("composite -> {}", path.display());
    }
    Ok(())
}
