use bevy::prelude::*;
use oc_root::craters::CraterType;
use oc_root::files::AssetDir;

const SIZE: u32 = 64;
const COLUMNS: u32 = 5;
const ROWS: u32 = 2;

#[derive(Debug, Resource)]
pub struct CraterSprites {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl CraterSprites {
    pub fn init(
        sprites: &AssetDir,
        assets: &AssetServer,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        Self {
            image: assets.load(sprites.join("craters.png")),
            layout: atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(SIZE),
                COLUMNS,
                ROWS,
                None,
                None,
            )),
        }
    }

    /// Sprite of crater `index` of `type_`, scaled to `size` pixels (diameter)
    pub fn sprite(&self, type_: CraterType, index: usize, size: f32) -> Sprite {
        let mut sprite = Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index: type_.row() * COLUMNS as usize + index % COLUMNS as usize,
            },
        );
        sprite.custom_size = Some(Vec2::splat(size));
        sprite
    }
}
