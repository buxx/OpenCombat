use std::{
    any::Any,
    path::{Path, PathBuf},
};

use battle_core::{config::ServerConfig, map::Map, types::WorldPoint};
use ggez::{
    graphics::{Color, DrawMode, DrawParam, Image, InstanceArray, MeshBuilder, Rect},
    Context, GameError, GameResult,
};
use image::{ImageBuffer, RgbaImage};
use oc_core::resources::Resources;
use rayon::prelude::*;

use crate::utils::qualified::ToQualified;

use super::qualified::Zoom;

pub fn ensure_map_dark_backgrounds(map: &Map) -> GameResult<PathBuf> {
    let resources = match Resources::new() {
        Ok(resources) => resources,
        Err(error) => return Err(GameError::ResourceLoadError(error.to_string())),
    };
    let bg_image_path_abs = resources.lib().join(
        map.background_image_path()
            .strip_prefix("/")
            .expect("Must start with /"),
    );
    let bg_image_hd_path_abs = resources.lib().join(
        map.background_image_path()
            .to_qualified(&Zoom::hd())
            .map_err(|error| {
                GameError::ResourceLoadError(format!(
                    "Background image source qualification error : {}",
                    error
                ))
            })?
            .strip_prefix("/")
            .expect("Must start with /"),
    );
    let bg_dark_image_path_abs = resources
        .cache_abs()
        .join(format!("{}__dark.png", map.name()));
    let bg_dark_image_path_rel = resources
        .cache_ggez()
        .join(format!("{}__dark.png", map.name()));
    let bg_dark_image_hd_path_abs = resources
        .cache_abs()
        .join(format!("{}__dark__HD.png", map.name()));

    if !bg_dark_image_path_abs.exists() {
        let mut bg_image = image::open(bg_image_path_abs)?.into_rgba8();
        bg_image
            .as_flat_samples_mut()
            .samples
            .par_chunks_mut(4)
            .for_each(|channels: &mut [u8]| channels[3] = 84);
        let mut dark_bg_image: RgbaImage =
            ImageBuffer::from_pixel(bg_image.width(), bg_image.height(), [0, 0, 0, 255].into());
        image::imageops::overlay(&mut dark_bg_image, &bg_image, 0, 0);
        dark_bg_image.save(bg_dark_image_path_abs)?;
    }

    if !bg_dark_image_hd_path_abs.exists() {
        let mut bg_image = image::open(bg_image_hd_path_abs)?.into_rgba8();
        bg_image
            .as_flat_samples_mut()
            .samples
            .par_chunks_mut(4)
            .for_each(|channels: &mut [u8]| channels[3] = 84);
        let mut dark_bg_image_hd: RgbaImage =
            ImageBuffer::from_pixel(bg_image.width(), bg_image.height(), [0, 0, 0, 255].into());
        image::imageops::overlay(&mut dark_bg_image_hd, &bg_image, 0, 0);
        dark_bg_image_hd.save(bg_dark_image_hd_path_abs)?;
    }

    Ok(bg_dark_image_path_rel)
}

pub fn ensure_dark(map_name: &str, image_path: &Path) -> GameResult<()> {
    let resources = match Resources::new() {
        Ok(resources) => resources,
        Err(error) => return Err(GameError::ResourceLoadError(error.to_string())),
    };
    let image_path_abs = resources.lib().join(
        image_path
            .strip_prefix("/")
            .expect("Given file must start with /"),
    );
    let file_name = image_path_abs
        .file_name()
        .ok_or(GameError::ResourceLoadError(format!(
            "Fail to ensure dark version of '{}' (determine file name)",
            image_path_abs.display()
        )))?
        .to_string_lossy()
        .to_string();
    let file_name_without_extension = PathBuf::from(file_name)
        .file_stem()
        .ok_or(GameError::ResourceLoadError(format!(
            "Fail to ensure dark version of '{}' (determine file name without extension)",
            image_path_abs.display()
        )))?
        .to_string_lossy()
        .to_string();
    let dark_image_path_abs = resources.cache_abs().join(format!(
        "{}__{}__dark.png",
        map_name, file_name_without_extension
    ));
    if !dark_image_path_abs.exists() {
        let mut image = image::open(&image_path_abs)?.into_rgba8();
        image
            .as_flat_samples_mut()
            .samples
            .par_chunks_mut(4)
            .for_each(|channels: &mut [u8]| channels[3] = 84);
        let mut dark_image: RgbaImage =
            ImageBuffer::from_pixel(image.width(), image.height(), [0, 0, 0, 255].into());
        image::imageops::overlay(&mut dark_image, &image, 0, 0);
        dark_image.save(dark_image_path_abs)?;
    }

    Ok(())
}

pub fn create_debug_terrain_batch(ctx: &mut Context, map: &Map) -> GameResult<InstanceArray> {
    let mut batch = InstanceArray::new(
        ctx,
        Image::from_path(ctx, map.terrain_image_path().display().to_string())?,
    );

    for tile in map.terrain_tiles() {
        let src_x = tile.tile_x as f32 * tile.relative_tile_width;
        let src_y = tile.tile_y as f32 * tile.relative_tile_height;
        let dest_x = tile.x as f32 * tile.tile_width as f32;
        let dest_y = tile.y as f32 * tile.tile_height as f32;
        batch.push(
            DrawParam::new()
                .src(Rect::new(
                    src_x,
                    src_y,
                    tile.relative_tile_width,
                    tile.relative_tile_height,
                ))
                .dest(WorldPoint::new(dest_x, dest_y).to_vec2()),
        );
    }

    Ok(batch)
}

pub fn create_debug_terrain_opacity_mesh_builder(
    map: &Map,
    config: &ServerConfig,
) -> GameResult<MeshBuilder> {
    let mut debug_terrain_opacity_mesh = MeshBuilder::new();
    for tile in map.terrain_tiles() {
        let dest_x = tile.x as f32 * tile.tile_width as f32;
        let dest_y = tile.y as f32 * tile.tile_height as f32;
        let color_modifier = config.terrain_tile_opacity(&tile.type_);
        debug_terrain_opacity_mesh.rectangle(
            DrawMode::fill(),
            Rect::new(
                dest_x,
                dest_y,
                tile.tile_width as f32,
                tile.tile_height as f32,
            ),
            Color {
                r: 0.4 - color_modifier,
                g: 0.4 - color_modifier,
                b: 0.4 - color_modifier,
                a: 1.0,
            },
        )?;
    }
    Ok(debug_terrain_opacity_mesh)
}
