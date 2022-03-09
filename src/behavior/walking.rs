use crate::{message::*, types::*};

pub fn entity_updates(entity: &ThreadSafeEntity, _path: &Vec<WorldPath>) -> Vec<EntityMessage> {
    let mut messages = vec![];

    // FIXME demo code, this will be in "update" code
    let current_point = entity.get_world_point();
    let new_point = WorldPoint::from((current_point.x + 1., current_point.y + 1.));
    let entity_message = EntityMessage::SetWorldPosition(new_point);
    messages.push(entity_message);

    messages
}
