use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;
use derive_more::Constructor;
use oc_root::files::AssetDir;
use oc_root::{
    side::{BySide, Side},
    sprites,
};

use crate::sprites::IntoAnimation;

// soldiers.png
#[derive(Debug, Resource)]
pub struct SoldierAnimations {
    sprite: Sprite,
    outline: Sprite,
    sides: BySide<SideAnimations>,
}

/// Animations of one side soldiers
#[derive(Debug)]
struct SideAnimations {
    stand_up: Handle<Animation>,
    walking: Handle<Animation>,
    running: Handle<Animation>,
    crawling: Handle<Animation>,
    prone: Handle<Animation>,
    dead_prone: Handle<Animation>,
    #[allow(unused)]
    hurt_prone: Handle<Animation>,
}

impl SideAnimations {
    /// Build side animations from its spritesheet rows (starting at `row`)
    fn new(spritesheet: &Spritesheet, animations: &mut Assets<Animation>, row: usize) -> Self {
        let mut strip = |row: usize, per_frame: u32| {
            animations.add(
                spritesheet
                    .create_animation()
                    .add_horizontal_strip(0, row, 2)
                    .set_duration(AnimationDuration::PerFrame(per_frame))
                    .build(),
            )
        };
        let stand_up = strip(row, 1000);
        let prone = strip(row + 3, 1000);
        let dead_prone = strip(row + 4, 1000);
        let hurt_prone = strip(row + 5, 1000);
        let mut full_row = |row: usize, per_frame: u32| {
            animations.add(
                spritesheet
                    .create_animation()
                    .add_row(row)
                    .set_duration(AnimationDuration::PerFrame(per_frame))
                    .build(),
            )
        };

        Self {
            stand_up,
            walking: full_row(row + 1, 500),
            running: full_row(row + 1, 250),
            crawling: full_row(row + 2, 500),
            prone,
            dead_prone,
            hurt_prone,
        }
    }
}

impl SoldierAnimations {
    pub fn init(
        sprites: &AssetDir,
        outline: &str,
        assets: &AssetServer,
        animations: &mut Assets<Animation>,
        atlas_layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        let image = assets.load(sprites.join("soldiers.png"));
        let spritesheet = Spritesheet::new(
            &image,
            sprites::individual::COLUMNS,
            sprites::individual::ROWS,
        );

        let sides = BySide::from_fn(|side| {
            let row = match side {
                Side::A => 0,
                Side::B => sprites::individual::SIDE_B_START_ROW,
            };
            SideAnimations::new(&spritesheet, animations, row)
        });

        let sprite = spritesheet
            .with_size_hint(
                sprites::individual::IMAGE_WIDTH,
                sprites::individual::IMAGE_HEIGHT,
            )
            .sprite(atlas_layouts);

        let outline = Sprite {
            image: assets.load(outline.to_string()),
            texture_atlas: sprite.texture_atlas.clone(),
            ..default()
        };

        Self {
            sprite,
            outline,
            sides,
        }
    }

    pub fn sprite(&self) -> Sprite {
        self.sprite.clone()
    }

    pub fn outline_sprite(&self) -> Sprite {
        self.outline.clone()
    }

    pub fn stand_up(&self, side: Side) -> Handle<Animation> {
        self.sides[side].stand_up.clone()
    }

    pub fn walking(&self, side: Side) -> Handle<Animation> {
        self.sides[side].walking.clone()
    }

    pub fn running(&self, side: Side) -> Handle<Animation> {
        self.sides[side].running.clone()
    }

    pub fn crawling(&self, side: Side) -> Handle<Animation> {
        self.sides[side].crawling.clone()
    }

    pub fn prone(&self, side: Side) -> Handle<Animation> {
        self.sides[side].prone.clone()
    }

    pub fn dead_prone(&self, side: Side) -> Handle<Animation> {
        self.sides[side].dead_prone.clone()
    }

    #[allow(unused)]
    pub fn hurt_prone(&self, side: Side) -> Handle<Animation> {
        self.sides[side].hurt_prone.clone()
    }
}

#[derive(Debug, Constructor)]
pub struct SoldierAnimationInfos {
    side: Side,
    status: oc_individual::Status,
    gesture: oc_individual::Gesture,
}

impl IntoAnimation<SoldierAnimations> for SoldierAnimationInfos {
    fn animation(&self, animations: &SoldierAnimations) -> Handle<Animation> {
        let SoldierAnimationInfos {
            side,
            gesture,
            status,
        } = self;

        match status {
            oc_individual::Status::Operational | oc_individual::Status::Stunned(_) => {
                match gesture.body {
                    oc_individual::BodyGesture::StandUp(_) => animations.stand_up(*side),
                    oc_individual::BodyGesture::Walking(_) => animations.walking(*side),
                    oc_individual::BodyGesture::Running(_) => animations.running(*side),
                    oc_individual::BodyGesture::Crawling(_) => animations.crawling(*side),
                    oc_individual::BodyGesture::Prone(_) => animations.prone(*side),
                }
            }
            oc_individual::Status::Dead => animations.dead_prone(*side),
        }
    }
}
