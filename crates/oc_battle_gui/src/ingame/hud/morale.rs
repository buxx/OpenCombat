use bevy::prelude::*;
use oc_world::morale::Morale;

const HEIGHT: f32 = 10.;
const BG: Color = Color::srgb(0.12, 0.12, 0.12);
const COLOR_A: Color = Color::srgb(0.2, 0.4, 1.);
const COLOR_B: Color = Color::srgb(1., 0.1, 0.1);

#[derive(Debug, Event)]
pub struct UpdateMorale(pub Morale);

#[derive(Component, Default, Clone)]
pub struct MoraleFillA;

#[derive(Component, Default, Clone)]
pub struct MoraleFillB;

/// Rectangle split in two: left half is side A morale, right half is side B morale
pub fn morale() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: px(HEIGHT),
            column_gap: px(1),
            flex_shrink: 0.,
        }
        Children [
            Node {
                width: percent(50),
                height: percent(100),
                justify_content: JustifyContent::FlexEnd,
            }
            BackgroundColor(BG)
            Children [
                MoraleFillA
                Node {
                    width: percent(100),
                    height: percent(100),
                }
                BackgroundColor(COLOR_A)
            ]
            --
            Node {
                width: percent(50),
                height: percent(100),
                justify_content: JustifyContent::FlexStart,
            }
            BackgroundColor(BG)
            Children [
                MoraleFillB
                Node {
                    width: percent(100),
                    height: percent(100),
                }
                BackgroundColor(COLOR_B)
            ]
        ]
    }
}

pub fn on_update_morale(
    event: On<UpdateMorale>,
    mut a: Single<&mut Node, (With<MoraleFillA>, Without<MoraleFillB>)>,
    mut b: Single<&mut Node, (With<MoraleFillB>, Without<MoraleFillA>)>,
) {
    a.width = percent(event.0.a.clamp(0., 100.));
    b.width = percent(event.0.b.clamp(0., 100.));
}
