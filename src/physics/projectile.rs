use crate::physics::hit::determine_hit_type;
use crate::physics::visibility::Visibility;
use crate::physics::HitType;
use crate::scene::item::{SceneItem, SceneItemModifier, Side};
use crate::scene::main::MainStateModifier;
use crate::{Message, ScenePoint};

pub fn bullet_fire(
    frame_i: u32,
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
        messages.extend(
            match determine_hit_type(visibility, from_scene_item, to_scene_item) {
                HitType::Deadly => {
                    vec![Message::SceneItemMessage(
                        to_scene_item.id,
                        SceneItemModifier::Death,
                    )]
                }
                HitType::Incapacity => {
                    vec![Message::SceneItemMessage(
                        to_scene_item.id,
                        SceneItemModifier::Incapacity,
                    )]
                }
                HitType::Missed => {
                    vec![]
                }
            },
        )
    }

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
