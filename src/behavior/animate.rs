use crate::behavior::order::Order;
use crate::behavior::ItemBehavior;
use crate::config::MOVE_TO_REACHED_WHEN_DISTANCE_INFERIOR_AT;
use crate::scene::item::{ItemState, SceneItem, SceneItemModifier};
use crate::util::velocity_for_behavior;
use std::f32::consts::FRAC_PI_2;

pub fn digest_next_order(scene_item: &SceneItem) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    // TODO: Compute here if it possible (fear, compatible with current order, etc)
    if let Some(next_order) = &scene_item.next_order {
        match next_order {
            Order::MoveTo(_) => {
                scene_item_modifiers.push(SceneItemModifier::SwitchToNextOrder);
            }
            Order::MoveFastTo(_) => scene_item_modifiers.push(SceneItemModifier::SwitchToNextOrder),
            Order::HideTo(_) => scene_item_modifiers.push(SceneItemModifier::SwitchToNextOrder),
        }
    }

    scene_item_modifiers
}

pub fn digest_current_order(scene_item: &SceneItem) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    // TODO: here, compute state according to order. Ex: if too much fear, move order do not produce walking state
    if let Some(current_order) = &scene_item.current_order {
        match current_order {
            Order::MoveTo(move_to_scene_point) => {
                // FIXME BS NOW: Change order only if it is not the current order
                scene_item_modifiers.push(SceneItemModifier::ChangeState(ItemState::new(
                    ItemBehavior::MoveTo(*move_to_scene_point),
                )))
            }
            Order::MoveFastTo(move_to_scene_point) => {
                // FIXME BS NOW: Change order only if it is not the current order
                scene_item_modifiers.push(SceneItemModifier::ChangeState(ItemState::new(
                    ItemBehavior::MoveFastTo(*move_to_scene_point),
                )))
            }
            Order::HideTo(move_to_scene_point) => {
                // FIXME BS NOW: Change order only if it is not the current order
                scene_item_modifiers.push(SceneItemModifier::ChangeState(ItemState::new(
                    ItemBehavior::HideTo(*move_to_scene_point),
                )))
            }
        }
    }

    scene_item_modifiers
}

pub fn digest_current_behavior(scene_item: &SceneItem) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    match scene_item.state.current_behavior {
        ItemBehavior::Standing => {}
        ItemBehavior::MoveTo(going_to_scene_point)
        | ItemBehavior::MoveFastTo(going_to_scene_point)
        | ItemBehavior::HideTo(going_to_scene_point) => {
            // Note: angle computed by adding FRAC_PI_2 because sprites are north oriented
            scene_item_modifiers.push(SceneItemModifier::ChangeDisplayAngle(
                f32::atan2(
                    going_to_scene_point.y - scene_item.position.y,
                    going_to_scene_point.x - scene_item.position.x,
                ) + FRAC_PI_2,
            ));

            // Check if scene_point reached
            let distance = going_to_scene_point.distance(scene_item.position);
            let velocity = velocity_for_behavior(&scene_item.state.current_behavior)
                .expect("must have velocity here");
            if distance < MOVE_TO_REACHED_WHEN_DISTANCE_INFERIOR_AT * velocity {
                scene_item_modifiers.push(SceneItemModifier::ChangeState(ItemState::new(
                    ItemBehavior::Standing,
                )));

                // Test if reached destination is from an order. If it is, switch to next order.
                if let Some(current_order) = &scene_item.current_order {
                    match current_order {
                        Order::MoveTo(move_to_scene_point)
                        | Order::MoveFastTo(move_to_scene_point)
                        | Order::HideTo(move_to_scene_point) => {
                            if *move_to_scene_point == going_to_scene_point {
                                scene_item_modifiers.push(SceneItemModifier::SwitchToNextOrder);
                            }
                        }
                    }
                }
            }
        }
    }

    scene_item_modifiers
}
