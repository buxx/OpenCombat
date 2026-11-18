use bevy::prelude::*;
use oc_world::decor::DecorTileset;

#[derive(Debug, Resource)]
pub struct DecorSprites {
    pub tileset: DecorTileset,
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl DecorSprites {
    pub fn init(
        image: &str,
        tileset: DecorTileset,
        assets: &AssetServer,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(tileset.tile_width, tileset.tile_height),
            tileset.columns,
            tileset.rows,
            None,
            None,
        );

        Self {
            image: assets.load(image.to_string()),
            layout: atlas_layouts.add(layout),
            tileset,
        }
    }

    pub fn sprite(&self, tile: u32) -> Sprite {
        Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index: tile as usize,
            },
        )
    }
}
