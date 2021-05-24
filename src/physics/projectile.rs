use crate::physics::HitType;
use crate::scene::item::{SceneItem, Side};
use crate::scene::main::MainStateModifier;
use crate::{Message, ScenePoint};

pub fn bullet_fire(
    frame_i: u32,
    from_scene_point: &ScenePoint,
    from_scene_item: &SceneItem,
    to_scene_point: &ScenePoint,
    target_scene_item: Option<&SceneItem>,
    hit_type: &HitType,
) -> Vec<Message> {
    let mut messages: Vec<Message> = vec![];

    messages.push(Message::MainStateMessage(MainStateModifier::NewProjectile(
        Projectile::new(
            from_scene_point.clone(),
            to_scene_point.clone(),
            frame_i,
            frame_i + 5, // FIXME: depending distance
            from_scene_item.side.clone(),
        ),
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
