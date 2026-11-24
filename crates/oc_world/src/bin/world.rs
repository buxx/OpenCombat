use oc_root::format::RkyvFile;
use std::path::{Path, PathBuf};

use ::image::{ImageBuffer, Rgba};
use anyhow::Context;
use clap::Parser;
use oc_mod::Mod;
use oc_root::{
    DEFAULT_FIGHT_DURATION, HUD_MINIMAP_SIZE, WorldConfig, battle::BattlePhase, side::BySide,
};
use oc_utils::image;
use oc_world::{reader, snapshot::Snapshot, support::Supports, terrain::Terrain};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Folder which contain (or already contain) world
    #[clap()]
    pub path: PathBuf,

    /// Path to the mod folder (required if snapshot used)
    #[clap(long)]
    pub r#mod: Option<PathBuf>,

    /// File path to the snapshot file to initialize
    #[clap(long)]
    pub snapshot: Option<PathBuf>,

    /// World width in tiles (required if initializing new world)
    #[clap(long)]
    pub width: Option<u64>,

    /// World height in tiles (required if initializing new world)
    #[clap(long)]
    pub height: Option<u64>,

    /// Tile size (in pixel)
    #[clap(long, default_value = "5")]
    pub tile_size: u64,

    /// Terrain tsx source file
    #[clap(long)]
    pub terrain_tsx: Option<PathBuf>,

    /// Terrain png source file
    #[clap(long)]
    pub terrain_png: Option<PathBuf>,

    /// Decor tsx source file
    #[clap(long)]
    pub decor_tsx: Option<PathBuf>,

    /// Decor png source file
    #[clap(long)]
    pub decor_png: Option<PathBuf>,

    /// Print analysis informations
    #[clap(short, long)]
    pub verbose: bool,

    /// Replace files like terrain.tsx, snaphost, etc
    #[clap(long, action)]
    pub replace: bool,
}

fn copy(from: &PathBuf, to: &PathBuf, name: &str, replace: bool) -> Result<(), anyhow::Error> {
    let exist = std::fs::exists(to).context(format!("Test if {} exists", to.display()))?;

    if !exist || replace {
        tracing::info!("Copy {} ({} -> {})", name, from.display(), to.display());
        std::fs::copy(from, to).context(format!("Copy {}", name))?;
    } else {
        tracing::info!("{} already exist", name);
    }

    Ok(())
}

macro_rules! replace {
    ($string:expr, $pattern:expr, $value:expr) => {
        let start = $string
            .find($pattern)
            .context(format!("Replace '{}' in world.tmx: not found", $pattern))?;
        let end = start + $pattern.len();
        $string.replace_range(start..end, $value);
    };
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    setup_logging(args.verbose)?;

    std::fs::create_dir_all(&args.path).context(format!(
        "Create (if not exist) folder {}",
        args.path.display()
    ))?;

    let f = args.replace;

    let background_path = args.path.join("background.png");
    let interiors_path = args.path.join("interiors.png");
    let (width, height) = background(&args, &background_path)?;
    let w = WorldConfig::new(width, height);

    tracing::info!("Assuming width of {} (tiles)", width);
    tracing::info!("Assuming height of {} (tiles)", height);
    interiors(width, height, args.tile_size, &interiors_path)?;

    let terrain_tsx_tpl_path = PathBuf::from("templates/world/terrain.tsx");
    let terrain_tsx_path = args.path.join("terrain.tsx");
    copy(&terrain_tsx_tpl_path, &terrain_tsx_path, "terrain.tsx", f)?;

    let height_tsx_tpl_path = PathBuf::from("templates/world/height.tsx");
    let height_tsx_path = args.path.join("height.tsx");
    copy(&height_tsx_tpl_path, &height_tsx_path, "height.tsx", f)?;

    let height_png_tpl_path = PathBuf::from("templates/world/height.png");
    let height_png_path = args.path.join("height.png");
    copy(&height_png_tpl_path, &height_png_path, "height.png", f)?;

    let terrain_png_tpl_path = PathBuf::from("templates/world/terrain.png");
    let terrain_png_path = args.path.join("terrain.png");
    copy(&terrain_png_tpl_path, &terrain_png_path, "terrain.png", f)?;

    let decor_tsx_tpl_path = PathBuf::from("templates/world/decor.tsx");
    let decor_tsx_path = args.path.join("decor.tsx");
    copy(&decor_tsx_tpl_path, &decor_tsx_path, "decor.tsx", f)?;

    let decor_png_tpl_path = PathBuf::from("templates/world/decor.png");
    let decor_png_path = args.path.join("decor.png");
    copy(&decor_png_tpl_path, &decor_png_path, "decor.png", f)?;

    let meta_tpl_path = PathBuf::from("templates/world/meta.toml");
    let meta_path = args.path.join("meta.toml");
    copy(&meta_tpl_path, &meta_path, "meta.toml", f)?;

    let world_tpl_path = PathBuf::from("templates/world/world.tmx");
    let world_path = args.path.join("world.tmx");
    world(&world_tpl_path, &world_path, width, height, args.tile_size)?;

    minimaps(&w, &args.path, f)?;

    if let Some(mod_) = &args.r#mod {
        let mod_ = Mod::load(mod_, None).context(format!("Load cache from {}", mod_.display()))?;
        analyze_terrain(&w, &terrain_tsx_path, &mod_)?;
    }

    if let Some(snapshot) = &args.snapshot {
        let Some(mod_) = &args.r#mod else {
            anyhow::bail!("You must give mod folder path to generate snapshot");
        };
        let mod_ = Mod::load(mod_, None).context(format!("Load cache from {}", mod_.display()))?;
        snapshot_(w, &args.path, snapshot, &mod_, args.replace)?;
    }

    Ok(())
}

fn minimaps(w: &WorldConfig, dir: &Path, replace: bool) -> Result<(), anyhow::Error> {
    let minimap = dir.join(oc_world::load::MINIMAP_FILE);
    let hud_minimap = dir.join(oc_world::load::HUD_MINIMAP_FILE);
    if !replace && minimap.exists() && hud_minimap.exists() {
        tracing::info!("Minimaps already exist");
        return Ok(());
    }

    let background_path = dir.join("background.png");
    let background =
        ::image::open(&background_path).context(format!("Open {}", background_path.display()))?;

    for (path, width, height) in [
        (
            minimap,
            w.minimap_width_pixels() as u32,
            w.minimap_height_pixels() as u32,
        ),
        (hud_minimap, HUD_MINIMAP_SIZE, HUD_MINIMAP_SIZE),
    ] {
        if path.exists() && !replace {
            tracing::info!("{} already exists", path.display());
            continue;
        }
        tracing::info!("Generate {} ({width}x{height})", path.display());
        ::image::imageops::resize(
            &background,
            width,
            height,
            ::image::imageops::FilterType::Gaussian,
        )
        .save(&path)
        .context(format!("Save {}", path.display()))?;
    }

    Ok(())
}

fn analyze_terrain(w: &WorldConfig, path: &PathBuf, mod_: &Mod) -> Result<(), anyhow::Error> {
    let terrain =
        Terrain::load(path, w.clone(), mod_).context(format!("Load terrain {}", path.display()))?;
    tracing::debug!("Terrain colums: {}", terrain.columns());
    tracing::debug!("Terrain rows: {}", terrain.rows());

    let mut natures: Vec<(String, u32)> = terrain
        .natures
        .iter()
        .map(|(n, i)| (mod_.nature(*n).name.clone(), *i))
        .collect();
    natures.sort_by_key(|(_, i)| *i);

    for (index, nature) in natures {
        tracing::debug!("Terrain index {index} is: {nature}");
    }

    Ok(())
}

fn snapshot_(
    w: WorldConfig,
    map: &Path,
    path: &PathBuf,
    mod_: &Mod,
    replace: bool,
) -> Result<(), anyhow::Error> {
    if std::fs::exists(path).context(format!("Test if {} exists", path.display()))? {
        if replace {
            std::fs::remove_file(path).context(format!("Remove file {}", path.display()))?;
        } else {
            tracing::warn!(
                "{} already exist and --replace not set, skip snapshot generation",
                path.display()
            );
            return Ok(());
        }
    };

    tracing::info!("Initialize snapshot ({})", path.display());
    let mut snapshot = Snapshot::new(
        w,
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        BattlePhase::Deployment,
        DEFAULT_FIGHT_DURATION,
        None,
        BySide::default(),
        Supports::default(),
        vec![],
        vec![],
        Default::default(),
    );

    tracing::info!("Update snapshot tiles");
    let map = reader::MapReader::new(map)?;
    let tiles = map.tiles(&snapshot.w, mod_)?;
    snapshot.altered_tiles = vec![None; tiles.len()];
    snapshot.smoke = vec![oc_utils::percent::Percent::ZERO; tiles.len()];
    snapshot.tiles = tiles;
    snapshot.decor = map.decor()?;

    tracing::info!("Save snapshot");
    let snapshot = snapshot.save(path);
    snapshot.context(format!("Save snapshot ({})", path.display()))?;

    Ok(())
}

fn world(
    world_tpl_path: &PathBuf,
    world_path: &PathBuf,
    width: u64,
    height: u64,
    tile_size: u64,
) -> Result<(), anyhow::Error> {
    let mut world = match std::fs::read_to_string(world_path) {
        Ok(_) => {
            tracing::info!("{} already exists", world_path.display());
            return Ok(());
        }
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => {
                tracing::info!("Create {}", world_path.display());
                std::fs::copy(world_tpl_path, world_path).context(format!(
                    "Copy {} to {}",
                    world_tpl_path.display(),
                    world_path.display()
                ))?;
                std::fs::read_to_string(world_path)
                    .context(format!("Read {}", world_path.display()))?
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Can't read {}: {}",
                    world_path.display(),
                    error
                ));
            }
        },
    };

    let widthpx = width * tile_size;
    let heightpx = width * tile_size;

    replace!(world, "{{width}}", &width.to_string());
    replace!(world, "{{height}}", &height.to_string());
    replace!(world, "{{terrain_width}}", &width.to_string());
    replace!(world, "{{terrain_height}}", &height.to_string());
    replace!(world, "{{decor_width}}", &width.to_string());
    replace!(world, "{{decor_height}}", &height.to_string());
    replace!(world, "{{height_width}}", &width.to_string());
    replace!(world, "{{height_height}}", &height.to_string());
    replace!(world, "{{tile_width}}", &tile_size.to_string());
    replace!(world, "{{tile_height}}", &tile_size.to_string());
    replace!(world, "{{background_width}}", &(widthpx).to_string());
    replace!(world, "{{background_height}}", &(heightpx).to_string());
    replace!(world, "{{interiors_width}}", &(widthpx).to_string());
    replace!(world, "{{interiors_height}}", &(heightpx).to_string());

    let terrain = (0..height)
        .map(|_| vec!["1"; width as usize].join(","))
        .collect::<Vec<String>>()
        .join(",\n");
    replace!(world, "{{terrain}}", &terrain.to_string());

    let height_ = (0..height)
        .map(|_| vec!["2021"; width as usize].join(","))
        .collect::<Vec<String>>()
        .join(",\n");
    replace!(world, "{{height_}}", &height_.to_string());

    let decor = (0..height)
        .map(|_| vec!["0"; width as usize].join(","))
        .collect::<Vec<String>>()
        .join(",\n");
    replace!(world, "{{decor}}", &decor.to_string());

    tracing::info!("Write world.tmx content");
    std::fs::write(world_path, world)
        .context(format!("Write content in {}", world_path.display()))?;

    Ok(())
}

fn background(args: &Args, path: &PathBuf) -> Result<(u64, u64), anyhow::Error> {
    let (width, height) = match std::fs::exists(path)
        .context(format!("Test if {} exists", path.display()))?
    {
        true => {
            let (width, height) = image::get_png_dimensions(path).unwrap(); // TODO

            if let Some(width_) = args.width {
                if width_ * args.tile_size != width as u64 {
                    anyhow::bail!(
                        "You provide --width and/or --height parameters but one or both are different from current background.png size"
                    );
                }

                if !(width as u64).is_multiple_of(args.tile_size) {
                    anyhow::bail!("Background width is not divisible by given tile size");
                }
            }
            if let Some(height_) = args.height {
                if height_ * args.tile_size != height as u64 {
                    anyhow::bail!(
                        "You provide --width and/or --height parameters but one or both are different from current background.png size"
                    );
                }

                if !(height as u64).is_multiple_of(args.tile_size) {
                    anyhow::bail!("Background widheightth is not divisible by given tile size");
                }
            }

            tracing::info!("Background image already exist");
            (
                width as u64 / args.tile_size,
                height as u64 / args.tile_size,
            )
        }
        false => {
            let (Some(width), Some(height)) = (args.width, args.height) else {
                anyhow::bail!(
                    "You must provide --width and --height parameters to permit initialize background.png"
                );
            };
            let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(
                (width * args.tile_size) as u32,
                (height * args.tile_size) as u32,
            );

            tracing::info!("Create background image");
            img.save(&path)
                .context(format!("Create {}", path.display()))
                .context(format!("Write {} file", path.display()))?;

            (width, height)
        }
    };

    Ok((width, height))
}

fn interiors(width: u64, height: u64, tile_size: u64, path: &PathBuf) -> Result<(), anyhow::Error> {
    match std::fs::exists(path).context(format!("Test if {} exists", path.display()))? {
        true => {
            let (width_, height_) = image::get_png_dimensions(path).unwrap(); // TODO
            if width_ as u64 != width * tile_size {
                anyhow::bail!("interiors.png size don't match with background");
            }

            if !(width_ as u64).is_multiple_of(tile_size) {
                anyhow::bail!("interiors.png width is not divisible by given tile size");
            }

            if height_ as u64 != height * tile_size {
                anyhow::bail!("interiors.png size don't match with background");
            }

            if !(height_ as u64).is_multiple_of(tile_size) {
                anyhow::bail!("interiors.png height is not divisible by given tile size");
            }

            tracing::info!("interiors.png image already exist");
        }
        false => {
            let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
                ImageBuffer::new((width * tile_size) as u32, (height * tile_size) as u32);

            tracing::info!("Create interiors.png image");
            img.save(&path)
                .context(format!("Create {}", path.display()))
                .context(format!("Write {} file", path.display()))?;
        }
    };

    Ok(())
}

fn setup_logging(verbose: bool) -> Result<(), anyhow::Error> {
    let default_directive = match verbose {
        true => LevelFilter::DEBUG,
        false => LevelFilter::INFO,
    };

    tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(default_directive.into())
                .from_env()?,
        )
        .init();
    Ok(())
}
