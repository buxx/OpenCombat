use std::{
    io,
    path::PathBuf,
    sync::{Arc, atomic::AtomicU32},
    time::Instant,
};

use derive_more::Constructor;
use oc_geo::region::WorldRegionIndex;
use oc_mod::Mod;
use oc_projectile::NextProjectileId;
use oc_root::{WorldConfig, files, ids::Ids, side::BySide};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use thiserror::Error;

use crate::{
    World,
    cache::{self, CacheRegionBackgroundError},
    flag::Flags,
    meta::Meta,
    navmesh::{Walls, navmesh},
    reader::{MapReader, MapReaderError},
    snapshot::Snapshot,
    spawn::SpawnZone,
    utils::image::crop::crop_polygon,
    visibility::Visibilities,
};

pub const MINIMAP_FILE: &str = "minimap.png";
pub const HUD_MINIMAP_FILE: &str = "hud_minimap.png";

#[derive(Debug, Constructor)]
pub struct WorldLoader {
    w: WorldConfig,
    mod_: Mod,
    world: PathBuf,
    cache: files::Roots,
}

impl WorldLoader {
    pub fn load(&self, ids: &Ids, snapshot: Snapshot, map: &MapReader) -> Result<World, Error> {
        tracing::info!("Check world {}", self.world.display());
        self.check()?;

        tracing::info!("Load world meta {}", self.world.meta().display());
        let meta = Meta::from_file(&self.world.meta()).map_err(MetaError::Load)?;

        // TODO: centralize caching at server startup
        self.cache(&meta, self.w.region_width(), self.w.region_height(), map)?;
        tracing::debug!("Cache finished");

        let w = self.w.clone();
        let mod_ = self.mod_.clone();
        let altered_tiles = snapshot.altered_tiles;
        let mut tiles = snapshot.tiles;
        // Altered tiles update matching tiles
        for altered in altered_tiles.iter().flatten() {
            if let Some(tile) = tiles.get_mut(altered.i.0 as usize) {
                tile.alter(altered, &mod_);
            }
        }
        let decor = snapshot.decor;
        let craters = snapshot.craters;
        let smoke = snapshot.smoke;
        let smoke_clouds = snapshot.smoke_clouds.into_iter().map(Into::into).collect();
        let individuals = snapshot.individuals;
        let visibilities = Visibilities::empty(individuals.len());
        let squads = snapshot.squads;
        let uuids = snapshot.uuids;
        tracing::debug!("Build projectile ids");
        let projectiles = snapshot
            .projectiles
            .into_iter()
            .map(|projectile| (ids.next_projectile_id(), projectile))
            .collect();
        let phase = snapshot.phase;
        let duration = snapshot.duration;
        let start = snapshot.start.map(|start| {
            let elapsed = start.elapsed().unwrap_or_default();
            Instant::now()
                .checked_sub(elapsed)
                .unwrap_or_else(Instant::now)
        });
        let spawns = snapshot.spawns;
        let supports = snapshot.supports;
        let pending_supports = snapshot
            .pending_supports
            .into_iter()
            .map(Into::into)
            .collect();
        let support_realized = snapshot
            .support_realized
            .into_iter()
            .map(|r| {
                let at = r.at.elapsed().unwrap_or_default();
                let at = Instant::now().checked_sub(at).unwrap_or_else(Instant::now);
                ((r.side, r.type_), at)
            })
            .collect();
        tracing::debug!("Build navmesh grid");
        let walls = tiles.as_walls(&mod_);

        tracing::debug!("Build navmesh (from grid with len {})", walls.len());
        let navmesh = navmesh(&w, &walls);

        let flags = map.flags()?;
        let interiors = map.interiors()?;
        let places = map.places()?;
        let spawn_zones = map.spawn_zones()?;
        let side_spawn_zones: BySide<Vec<SpawnZone>> = spawns.map(|names| {
            spawn_zones
                .iter()
                .filter(|s| names.contains(&s.name))
                .cloned()
                .collect()
        });
        let flags = Flags::from_control(flags, &side_spawn_zones.a, &side_spawn_zones.b);

        tracing::debug!("World built");
        Ok(World::new(
            w,
            mod_,
            meta,
            phase,
            duration,
            start,
            side_spawn_zones,
            spawn_zones,
            flags,
            supports,
            pending_supports,
            support_realized,
            tiles,
            altered_tiles,
            decor,
            craters,
            smoke,
            smoke_clouds,
            navmesh,
            visibilities,
            individuals,
            squads,
            projectiles,
            interiors,
            places,
            uuids,
        ))
    }

    // TODO: add checks
    fn check(&self) -> Result<(), Error> {
        self.check_meta().map_err(Error::Meta)?;
        self.check_background()
            .map_err(|e| Error::Background(self.world.background(), e))?;

        Ok(())
    }

    fn check_background(&self) -> Result<(), BackgroundError> {
        if !self.world.background().exists() {
            return Err(BackgroundError::NotFound);
        }

        if !self.world.background().is_file() {
            return Err(BackgroundError::NotAFile);
        }

        let (width, height) = image::image_dimensions(self.world.background())?;
        if width != self.w.world_width_pixels() as u32
            || height != self.w.world_height_pixels() as u32
        {
            return Err(BackgroundError::Dimensions(
                width,
                height,
                self.w.world_width_pixels() as u32,
                self.w.world_height_pixels() as u32,
            ));
        }

        Ok(())
    }

    fn check_meta(&self) -> Result<(), MetaError> {
        if !self.world.meta().exists() {
            return Err(MetaError::NotFound);
        }

        if !self.world.meta().is_file() {
            return Err(MetaError::NotAFile);
        }

        Ok(())
    }

    // TODO: centralize caching at server startup
    fn cache(
        &self,
        meta: &Meta,
        region_width: u64,
        region_height: u64,
        map: &MapReader,
    ) -> Result<(), CacheError> {
        tracing::info!("Check cache for {}", self.world.display());
        let files = files::Files::derived_only(self.cache.clone(), "", &meta.canonical());
        let archive = files.fs(files::File::WorldArchive);
        // Regions and interiors are written in the world derived folder
        let region = files.fs(files::File::Region {
            index: 0,
            width: region_width,
            height: region_height,
        });
        for path in [&archive, &region] {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let background = image::open(self.world.background())?;
        let interior = image::open(self.world.interiors())?;

        for minimap in [self.world.minimap(), self.world.hud_minimap()] {
            if !minimap.is_file() {
                return Err(CacheError::MinimapNotFound(minimap));
            }
        }

        self.cache_regions(region_width, region_height, &files, &background)?;
        self.cache_interiors(map, &files, &interior)?;

        if !std::fs::exists(&archive)? {
            tracing::info!("Caching {} to {}", &self.world.display(), archive.display());

            let file = std::fs::File::create(&archive)?;
            let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            let mut builder = tar::Builder::new(encoder);

            // Archive content at its root: clients extract it under their own world directory
            builder.append_dir_all(".", &self.world)?;
            builder.finish()?;
            tracing::info!("Finished cache for world ({})", archive.display());
        }

        Ok(())
    }

    fn cache_regions(
        &self,
        region_width: u64,
        region_height: u64,
        files: &files::Files,
        image: &image::DynamicImage,
    ) -> Result<(), CacheError> {
        let counter = Arc::new(AtomicU32::new(0));
        tracing::info!("Prepare cache for regions");
        (0..self.w.regions_count())
            .into_par_iter()
            .map(|i| {
                let i = WorldRegionIndex(i);
                let region = files.fs(files::File::Region {
                    index: i.0,
                    width: region_width,
                    height: region_height,
                });

                match region.exists() {
                    true => {
                        tracing::info!("Region cache {} already exists", region.display());
                        Ok(())
                    }
                    false => {
                        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        cache::cache_region_background(&self.w, &region, image, i)
                    }
                }
            })
            .collect::<Result<Vec<()>, CacheRegionBackgroundError>>()?;
        let cached = counter.load(std::sync::atomic::Ordering::Relaxed);
        let already = self.w.regions_count() - cached as u64;
        tracing::info!("{} cached, {} already cached", cached, already);
        Ok(())
    }

    fn cache_interiors(
        &self,
        map: &MapReader,
        files: &files::Files,
        image: &image::DynamicImage,
    ) -> Result<(), CacheError> {
        let interiors = map.interiors()?;
        tracing::info!("Prepare cache for interiors");

        let mut cached = 0;
        for interior in &interiors {
            let path = files.fs(files::File::Interior(interior.id));

            if !std::fs::exists(&path).map_err(|e| {
                CacheError::Interior(format!(
                    "Error when reading interior {} file {}: {}",
                    interior.id,
                    path.display(),
                    e
                ))
            })? {
                let image = crop_polygon(image, &interior.points).map_err(|e| {
                    CacheError::Interior(format!("Error when crop interior {}: {}", interior.id, e))
                })?;
                image.save(&path).map_err(|e| {
                    CacheError::Interior(format!(
                        "Error when save interior {} to {}: {}",
                        interior.id,
                        path.display(),
                        e
                    ))
                })?;
                cached += 1;
            }
        }

        let already = interiors.len() - cached as usize;
        tracing::info!("{} cached, {} already cached", cached, already);
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum BackgroundError {
    #[error("Not found")]
    NotFound,
    #[error("Is not a file")]
    NotAFile,
    #[error("Io: {0}")]
    Io(#[from] io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::error::ImageError),
    #[error("Dimensions: current {0}x{1}, required: {2}x{3}")]
    Dimensions(u32, u32, u32, u32),
}

#[derive(Debug, Error)]
pub enum MetaError {
    #[error("Not found")]
    NotFound,
    #[error("Is not a file")]
    NotAFile,
    #[error("Io: {0}")]
    Load(#[from] oc_root::format::FileError),
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Io: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image: {0}")]
    Image(#[from] image::error::ImageError),
    #[error("Region background: {0}")]
    RegionBackground(#[from] CacheRegionBackgroundError),
    #[error("Interior: {0}")]
    Interior(String),
    #[error("Minimap {0} not found")]
    MinimapNotFound(PathBuf),
    #[error("Map reader: {0}")]
    MapReader(#[from] MapReaderError),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Background error ({0}): {1}")]
    Background(PathBuf, BackgroundError),
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    #[error("Meta error: {0}")]
    Meta(#[from] MetaError),
    #[error("Map read: {0}")]
    MapReader(#[from] MapReaderError),
}

pub trait WorldPath {
    fn background(&self) -> PathBuf;
    fn interiors(&self) -> PathBuf;
    fn meta(&self) -> PathBuf;
    fn minimap(&self) -> PathBuf;
    fn hud_minimap(&self) -> PathBuf;
}

impl WorldPath for PathBuf {
    fn background(&self) -> PathBuf {
        self.join("background.png")
    }

    fn interiors(&self) -> PathBuf {
        self.join("interiors.png")
    }

    fn meta(&self) -> PathBuf {
        self.join("meta.toml")
    }

    fn minimap(&self) -> PathBuf {
        self.join(MINIMAP_FILE)
    }

    fn hud_minimap(&self) -> PathBuf {
        self.join(HUD_MINIMAP_FILE)
    }
}
