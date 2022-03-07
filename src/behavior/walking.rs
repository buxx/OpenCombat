use crate::{message::*, types::*};

pub fn entity_updates(
    entity: &ThreadSafeEntity,
    _destination: &WorldPosition,
) -> Vec<EntityMessage> {
    let mut messages = vec![];

    // FIXME demo code, this will be in "update" code
    let current_position = entity.get_world_position();
    let new_position = WorldPosition::from((
        current_position.x + WorldX::from(1.),
        current_position.y + WorldY::from(1.),
    ));
    let entity_message = EntityMessage::SetWorldPosition(new_position);
    messages.push(entity_message);

    messages
}
