use crate::behavior::order::Order;
use crate::behavior::ItemBehavior;
use crate::config::MOVE_TO_REACHED_WHEN_DISTANCE_INFERIOR_AT;
use crate::map::Map;
use crate::physics::path::find_path;
use crate::physics::util::{grid_point_from_scene_point, scene_point_from_grid_point};
use crate::scene::item::{ItemState, SceneItem, SceneItemModifier};
use crate::scene::main::MainState;
use crate::util::velocity_for_behavior;
use std::f32::consts::FRAC_PI_2;

pub fn digest_next_order(scene_item: &SceneItem, map: &Map) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    // TODO: Compute here if it possible (fear, compatible with current order, etc)
    if let Some(next_order) = &scene_item.next_order {
        match next_order {
            Order::MoveTo(move_to_scene_point)
            | Order::MoveFastTo(move_to_scene_point)
            | Order::HideTo(move_to_scene_point) => {
                if let Some(mut grid_path) = find_path(
                    &map,
                    &scene_item.grid_position,
                    &grid_point_from_scene_point(move_to_scene_point, &map),
                ) {
                    grid_path.drain(0..1);
                    let behavior = match next_order {
                        Order::MoveTo(_) => ItemBehavior::MoveTo(*move_to_scene_point, grid_path),
                        Order::MoveFastTo(_) => {
                            ItemBehavior::MoveFastTo(*move_to_scene_point, grid_path)
                        }
                        Order::HideTo(_) => ItemBehavior::HideTo(*move_to_scene_point, grid_path),
                    };
                    scene_item_modifiers.push(SceneItemModifier::SwitchToNextOrder);
                    scene_item_modifiers
                        .push(SceneItemModifier::ChangeState(ItemState::new(behavior)))
                } else {
                    eprintln!("No path found to given scene point {}", move_to_scene_point);
                };
            }
        }
    }

    scene_item_modifiers
}

pub fn digest_current_order(scene_item: &SceneItem, map: &Map) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];
    // TODO: here, compute state according to order. Ex: if too much fear, move order do not produce escape state
    scene_item_modifiers
}

pub fn digest_current_behavior(scene_item: &SceneItem, map: &Map) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    match &scene_item.state.current_behavior {
        ItemBehavior::Standing => {}
        ItemBehavior::MoveTo(_, grid_path)
        | ItemBehavior::MoveFastTo(_, grid_path)
        | ItemBehavior::HideTo(_, grid_path) => {
            if let Some(going_to_grid_point) = grid_path.first() {
                let going_to_scene_point = scene_point_from_grid_point(going_to_grid_point, &map);

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
                    scene_item_modifiers.push(SceneItemModifier::ReachMoveGridPoint);

                    // FIXME BS NOW: dans le code qui consomme le ReachMoveGridPoint
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
            } else {
                eprintln!("No grid point in grid path !")
            }
        }
    }

    scene_item_modifiers
}
