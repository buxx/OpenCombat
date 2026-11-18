use bevy::prelude::*;
use oc_root::alter::Alter;
use oc_root::files::AssetDir;

// altered.png
const SIZE: u32 = 30;
const COLUMNS: u32 = 6;

#[derive(Debug, Resource)]
pub struct AlteredSprites {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

impl AlteredSprites {
    pub fn init(
        sprites: &AssetDir,
        assets: &AssetServer,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        Self {
            image: assets.load(sprites.join("altered.png")),
            layout: atlas_layouts.add(TextureAtlasLayout::from_grid(
                UVec2::splat(SIZE),
                COLUMNS,
                1,
                None,
                None,
            )),
        }
    }

    pub fn sprite(&self, alter: Alter) -> Sprite {
        Sprite::from_atlas_image(
            self.image.clone(),
            TextureAtlas {
                layout: self.layout.clone(),
                index: Self::index(alter),
            },
        )
    }

    fn index(alter: Alter) -> usize {
        match alter {
            Alter::BrickDebris => 0,
            Alter::MixedDebris => 1,
            Alter::StoneDebris => 2,
            Alter::WoodDebris => 3,
            Alter::VegetalDebris => 4,
            Alter::Generic => 5,
        }
    }
}
