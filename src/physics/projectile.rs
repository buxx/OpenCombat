use crate::audio::Sound;
use crate::config::TARGET_FPS;
use crate::gameplay::weapon::WeaponCharacteristic;
use crate::physics::hit::determine_hit_type;
use crate::physics::visibility::Visibility;
use crate::physics::HitType;
use crate::scene::item::{SceneItem, SceneItemModifier, Side};
use crate::scene::main::MainStateModifier;
use crate::{FrameI, Message, ScenePoint};

pub fn bullet_fire(
    frame_i: FrameI,
    visibility: &Visibility,
    from_scene_item: &SceneItem,
    to_scene_item: Option<&SceneItem>,
) -> Vec<Message> {
    let mut messages: Vec<Message> = vec![];

    messages.push(Message::MainStateMessage(MainStateModifier::NewProjectile(
        Projectile::new(
            visibility.from_scene_point,
            visibility.to_scene_point,
            frame_i,
            frame_i + 5, // FIXME: depending distance
            from_scene_item.side.clone(),
        ),
    )));
    messages.push(Message::SceneItemMessage(
        from_scene_item.id,
        SceneItemModifier::SetLastBulletFire(frame_i),
    ));

    if let Some(to_scene_item) = to_scene_item {
        let (hit_value, max_hit, hit_type) =
            determine_hit_type(visibility, from_scene_item, to_scene_item);
        messages.extend(match &hit_type {
            HitType::Deadly => {
                vec![
                    Message::SceneItemMessage(to_scene_item.id, SceneItemModifier::Death),
                    Message::MainStateMessage(MainStateModifier::NewSound(Sound::Injured1)),
                ]
            }
            HitType::Incapacity => {
                vec![
                    Message::SceneItemMessage(to_scene_item.id, SceneItemModifier::Incapacity),
                    Message::MainStateMessage(MainStateModifier::NewSound(Sound::Injured1)),
                ]
            }
            HitType::Missed => {
                // TODO: Add reducer by big miss or near miss, distance, etc ...
                vec![Message::SceneItemMessage(
                    to_scene_item.id,
                    SceneItemModifier::IncrementUnderFire,
                )]
            }
        });
        messages.push(Message::MainStateMessage(MainStateModifier::NewDebugText(
            frame_i + TARGET_FPS / 2,
            to_scene_item.position,
            format!("{:.0}/{:.0}({:?})", hit_value, max_hit, hit_type),
        )))
    }

    messages.push(Message::MainStateMessage(MainStateModifier::NewSound(
        WeaponCharacteristic::new(&from_scene_item.weapon.type_).sound,
    )));

    messages
}

pub struct Projectile {
    pub from_scene_point: ScenePoint,
    pub to_scene_point: ScenePoint,
    pub start: u32,
    pub end: u32,
    pub side: Side,
}

impl Projectile {
    pub fn new(
        from_scene_point: ScenePoint,
        to_scene_point: ScenePoint,
        start: u32,
        end: u32,
        side: Side,
    ) -> Self {
        Self {
            from_scene_point,
            to_scene_point,
            start,
            end,
            side,
        }
    }
}
