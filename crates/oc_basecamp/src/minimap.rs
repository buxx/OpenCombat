use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage, imageops::FilterType};
use oc_game::folder_name;
use oc_root::{SPAWN_ZONE_NONE_OPACITY, side::Side};
use oc_world::{
    reader::MapReader,
    spawn::{SpawnZone, SpawnZoneName},
};

use crate::{
    battle::{Battle, MAPS_FOLDER, Scenario},
    mods::ModInfo,
};

const MINIMAPS_FOLDER: &str = "minimaps";
const MINIMAP_FILE: &str = "minimap.png";
const SIZE: u32 = 1024;

fn cache_dir(app_dir: &Path) -> PathBuf {
    oc_game::cache_dir(app_dir).join(MINIMAPS_FOLDER)
}

pub fn battle_path(app_dir: &Path, battle: &Battle) -> PathBuf {
    cache_dir(app_dir).join(format!("{}.png", folder_name(battle.map())))
}

pub fn scenario_path(app_dir: &Path, battle: &Battle, scenario: &Scenario, mod_: &str) -> PathBuf {
    cache_dir(app_dir)
        .join(folder_name(battle.name()))
        .join(folder_name(mod_))
        .join(format!("{}.png", folder_name(scenario.name())))
}

pub fn generate_all(
    app_dir: &Path,
    battles: &[Battle],
    mods: &[ModInfo],
    errors: &mut Vec<String>,
) {
    for battle in battles {
        if let Err(error) = generate(app_dir, battle, mods) {
            errors.push(format!("Minimaps of battle '{}': {error}", battle.name()));
        }
    }
}

fn generate(app_dir: &Path, battle: &Battle, mods: &[ModInfo]) -> Result<(), String> {
    let base = base(app_dir, battle)?;

    let missing: Vec<(&Scenario, &ModInfo, PathBuf)> = mods
        .iter()
        .flat_map(|m| battle.scenarios().iter().map(move |s| (s, m)))
        .map(|(s, m)| (s, m, scenario_path(app_dir, battle, s, &m.name)))
        .filter(|(_, _, path)| !path.is_file())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }

    let zones = MapZones::read(battle)?;
    for (scenario, mod_, path) in missing {
        let image = zones.paint(
            &base,
            [
                (&scenario.side(Side::A).start_spawn_zones, mod_.a_color),
                (&scenario.side(Side::B).start_spawn_zones, mod_.b_color),
            ],
        );
        save(&image, &path)?;
    }

    Ok(())
}

/// Generate (or regenerate) the minimap at `path` with given side A and B spawn zones
pub fn generate_with_zones(
    app_dir: &Path,
    battle: &Battle,
    mod_: &ModInfo,
    a_zones: &[SpawnZoneName],
    b_zones: &[SpawnZoneName],
    path: &Path,
) -> Result<(), String> {
    let base = base(app_dir, battle)?;
    let image = MapZones::read(battle)?;
    let image = image.paint(&base, [(a_zones, mod_.a_color), (b_zones, mod_.b_color)]);

    save(&image, path)
}

/// Battle map minimap (without spawn zones), generated in cache if missing
fn base(app_dir: &Path, battle: &Battle) -> Result<RgbaImage, String> {
    let battle_path = battle_path(app_dir, battle);
    if !battle_path.is_file() {
        let source = Path::new(MAPS_FOLDER).join(battle.map()).join(MINIMAP_FILE);
        let image =
            image::open(&source).map_err(|e| format!("Open '{}': {e}", source.display()))?;
        let image = image
            .resize_exact(SIZE, SIZE, FilterType::Triangle)
            .to_rgba8();
        save(&image, &battle_path)?;
        return Ok(image);
    }

    Ok(image::open(&battle_path)
        .map_err(|e| format!("Open '{}': {e}", battle_path.display()))?
        .to_rgba8())
}

struct MapZones {
    zones: Vec<SpawnZone>,
    size_pixels: (f32, f32),
}

impl MapZones {
    fn read(battle: &Battle) -> Result<Self, String> {
        let map_path = Path::new(MAPS_FOLDER).join(battle.map());
        let map = MapReader::new(&map_path).map_err(|e| e.to_string())?;

        Ok(Self {
            zones: map.spawn_zones().map_err(|e| e.to_string())?,
            size_pixels: map.size_pixels(),
        })
    }

    fn paint(&self, base: &RgbaImage, sides: [(&[SpawnZoneName], [u8; 3]); 2]) -> RgbaImage {
        let mut image = base.clone();
        for (names, color) in sides {
            for zone in self.zones.iter().filter(|z| names.contains(&z.name)) {
                fill_zone(&mut image, zone, self.size_pixels, color);
            }
        }

        image
    }
}

fn save(image: &RgbaImage, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Create '{}': {e}", parent.display()))?;
    }
    image
        .save(path)
        .map_err(|e| format!("Save '{}': {e}", path.display()))
}

// WARNING: AI generated
/// Fill the zone polygon (even-odd rule), blended with the image with the spawn zone opacity.
/// Zone points are in map pixels, scaled to the image size.
fn fill_zone(image: &mut RgbaImage, zone: &SpawnZone, map_size: (f32, f32), color: [u8; 3]) {
    let (width, height) = image.dimensions();
    let (sx, sy) = (width as f32 / map_size.0, height as f32 / map_size.1);
    let points: Vec<(f32, f32)> = zone.points.iter().map(|p| (p.x * sx, p.y * sy)).collect();
    if points.len() < 3 {
        return;
    }

    let min_y = points
        .iter()
        .map(|p| p.1)
        .fold(f32::MAX, f32::min)
        .max(0.)
        .floor() as u32;
    let max_y = points
        .iter()
        .map(|p| p.1)
        .fold(f32::MIN, f32::max)
        .min(height as f32 - 1.)
        .ceil() as u32;

    for y in min_y..=max_y {
        let scan_y = y as f32 + 0.5;
        // x positions where polygon edges cross the scan line
        let mut crossings: Vec<f32> = vec![];
        for (i, a) in points.iter().enumerate() {
            let b = points[(i + 1) % points.len()];
            if (a.1 <= scan_y) != (b.1 <= scan_y) {
                crossings.push(a.0 + (scan_y - a.1) / (b.1 - a.1) * (b.0 - a.0));
            }
        }
        crossings.sort_by(f32::total_cmp);

        for &[start, end] in crossings.as_chunks::<2>().0 {
            let from = start.max(0.).round() as u32;
            let to = (end.min(width as f32).round() as u32).min(width);
            for x in from..to {
                blend(image.get_pixel_mut(x, y), color);
            }
        }
    }
}

fn blend(pixel: &mut Rgba<u8>, color: [u8; 3]) {
    for (channel, color) in pixel.0.iter_mut().zip(color) {
        let mixed = *channel as f32 * (1. - SPAWN_ZONE_NONE_OPACITY)
            + color as f32 * SPAWN_ZONE_NONE_OPACITY;
        *channel = mixed.round() as u8;
    }
}
