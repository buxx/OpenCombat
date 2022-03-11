use crate::{message::*, types::*};

use super::Behavior;

pub fn entity_updates(entity: &ThreadSafeEntity, path: &WorldPaths) -> Vec<EntityMessage> {
    let mut messages = vec![];

    // There is a next point in path, go to it
    if let Some(point) = path.next_point() {
        let velocity = entity
            .get_behavior()
            .velocity()
            .expect("Entity behavior must have velocity when walking code called");
        let vector = (point.to_vec2() - entity.get_world_point().to_vec2()).normalize() * velocity;
        // Point reached
        if (entity.get_world_point().to_vec2() - point.to_vec2()).length() <= vector.length() {
            messages.push(EntityMessage::ReachBehaviorStep);
        // Movement required
        } else {
            let new_point = entity.get_world_point().apply(vector);
            let entity_message = EntityMessage::SetWorldPosition(new_point);
            messages.push(entity_message);
        }
    // There is no more points to reach
    } else {
        let entity_message = EntityMessage::SetBehavior(Behavior::Idle);
        messages.push(entity_message);
    }

    messages
}
