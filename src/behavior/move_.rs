use crate::{message::*, types::*};

use super::Behavior;

pub fn entity_updates(entity: &ThreadSafeEntity, path: &WorldPaths) -> Vec<EntityMessage> {
    let mut messages = vec![];

    let point = path.next_point().expect("Must have point in path");

    // There is a next point in path, go to it
    let velocity = entity
        .get_behavior()
        .velocity()
        .expect("Entity behavior must have velocity when move code called");
    let vector = (point.to_vec2() - entity.get_world_point().to_vec2()).normalize() * velocity;
    // Point reached
    if (entity.get_world_point().to_vec2() - point.to_vec2()).length() <= vector.length() {
        // If it is the last point, move is finished
        if path.is_last_point().expect("Must contain points") {
            messages.push(EntityMessage::SetBehavior(Behavior::Idle));
        } else {
            messages.push(EntityMessage::ReachBehaviorStep);
        }

    // Movement required
    } else {
        let new_point = entity.get_world_point().apply(vector);
        let entity_message = EntityMessage::SetWorldPosition(new_point);
        messages.push(entity_message);
    }

    messages
}
