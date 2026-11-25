use std::path::Path;

use bevy::prelude::*;
use oc_root::files::AssetDir;

pub const VEHICLES_PNG: &str = "vehicles.png";
const COLUMNS: u32 = 4;
const CHASSIS_COLUMN: u32 = 0;
const CHASSIS_SHADOW_COLUMN: u32 = 1;
const TURRET_COLUMN: u32 = 2;
const TURRET_SHADOW_COLUMN: u32 = 3;

#[derive(Debug, Resource)]
pub struct VehicleSprites {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl VehicleSprites {
    pub fn init(
        sprites: &AssetDir,
        image: &Path,
        rows: u32,
        assets: &AssetServer,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> anyhow::Result<Self> {
        let (width, _) = image::image_dimensions(image)?;
        let frame_size = width / COLUMNS;
        anyhow::ensure!(
            frame_size > 0,
            "{} too small ({width}px wide, {COLUMNS} columns expected)",
            image.display()
        );

        Ok(Self {
            image: assets.load(sprites.join(VEHICLES_PNG)),
            layout: atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(frame_size),
                COLUMNS,
                rows.max(1),
                None,
                None,
            )),
        })
    }

    pub fn chassis(&self, row: u8) -> Sprite {
        self.sprite(row, CHASSIS_COLUMN)
    }

    pub fn turret(&self, row: u8) -> Sprite {
        self.sprite(row, TURRET_COLUMN)
    }

    pub fn chassis_shadow(&self, row: u8) -> Sprite {
        self.sprite(row, CHASSIS_SHADOW_COLUMN)
    }

    pub fn turret_shadow(&self, row: u8) -> Sprite {
        self.sprite(row, TURRET_SHADOW_COLUMN)
    }

    fn sprite(&self, row: u8, column: u32) -> Sprite {
        Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index: (row as u32 * COLUMNS + column) as usize,
            },
        )
    }
}
