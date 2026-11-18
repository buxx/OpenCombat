use bevy::prelude::*;
use oc_root::{Wcfg, WcfgInto, geo::ScreenVec2};
use oc_utils::let_some;
use oc_world::flag::{FlagName, Flags, Ownership};

use crate::{
    ingame::draw::Z_FLAG,
    states::GuiFiles,
    utils::hover::{self, Hovered},
};

const SIZE: UVec2 = UVec2::new(50, 25);
const ROWS: u32 = 4;
const DEFAULT_ALPHA: f32 = 0.4;
const HOVER_ALPHA: f32 = 0.2; // When cursor hover sprite

#[derive(Debug, Event)]
pub struct UpdateFlags(pub Flags);

#[derive(Debug, Component)]
pub struct FlagSprite(FlagName);

#[derive(Debug, Component)]
pub struct FlagText(FlagName);

fn index(ownership: &Ownership) -> usize {
    match ownership {
        Ownership::A => 0,
        Ownership::B => 1,
        Ownership::Both => 2,
        Ownership::Nobody => 3,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn on_update_flags(
    event: On<UpdateFlags>,
    mut commands: Commands,
    w: Res<Wcfg>,
    files: Res<GuiFiles>,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    sprites: Query<Entity, With<FlagSprite>>,
    mut layout: Local<Option<Handle<TextureAtlasLayout>>>,
) {
    let_some!(w = &w.0, return);
    let_some!(files = &files.0, return);
    let layout = layout
        .get_or_insert_with(|| {
            layouts.add(TextureAtlasLayout::from_grid(SIZE, 1, ROWS, None, None))
        })
        .clone();

    for entity in &sprites {
        commands.entity(entity).despawn();
    }

    for (flag, ownership) in event.0.iter() {
        let index = index(ownership);

        let a: ScreenVec2 = flag.corners[0].into_(w);
        let b: ScreenVec2 = flag.corners[2].into_(w);
        let center = Vec2::new((a.x + b.x) / 2., (a.y + b.y) / 2.);

        let image = assets.load(files.sprites().join("flags.png"));
        commands
            .spawn((
                Name::new(flag.name.0.clone()),
                Sprite {
                    color: Color::WHITE.with_alpha(DEFAULT_ALPHA),
                    ..Sprite::from_atlas_image(
                        image,
                        TextureAtlas {
                            layout: layout.clone(),
                            index,
                        },
                    )
                },
                Transform::from_xyz(center.x, center.y, Z_FLAG),
                FlagSprite(flag.name.clone()),
                Pickable::default(),
                Hovered::default(),
                children![(
                    FlagText(flag.name.clone()),
                    Text2d::new(flag.name.0.clone()),
                    TextFont {
                        font_size: bevy::text::FontSize::Px(10.),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0., 0., 0.01),
                )],
            ))
            .observe(hover::over)
            .observe(hover::out);
    }
}

pub fn hovered(
    mut flags: Query<(&FlagSprite, &Hovered, &mut Sprite), Changed<Hovered>>,
    mut texts: Query<(&FlagText, &mut TextColor)>,
) {
    for (flag, hovered, mut sprite) in &mut flags {
        let (sprite_alpha, text_alpha) = if **hovered {
            (HOVER_ALPHA, HOVER_ALPHA)
        } else {
            (DEFAULT_ALPHA, 1.)
        };

        sprite.color.set_alpha(sprite_alpha);
        let_some!((_, mut c) = texts.iter_mut().find(|(t, _)| t.0 == flag.0), continue);
        c.0.set_alpha(text_alpha);
    }
}
