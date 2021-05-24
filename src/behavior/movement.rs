use crate::behavior::order::Order;
use crate::behavior::ItemBehavior;
use crate::config::MOVE_TO_REACHED_WHEN_DISTANCE_INFERIOR_AT;
use crate::map::Map;
use crate::physics::path::find_path;
use crate::physics::util::{grid_point_from_scene_point, scene_point_from_grid_point};
use crate::scene::item::{SceneItem, SceneItemModifier};
use crate::util::{angle, velocity_for_behavior};
use crate::{GridPath, ScenePoint};
use std::f32::consts::FRAC_PI_2;

pub fn digest_next_move_order(
    scene_item: &SceneItem,
    move_to_scene_point: ScenePoint,
    order: &Order,
    map: &Map,
) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    // TODO: Compute here if it possible (fear, compatible with current order, etc)
    if let Some(mut grid_path) = find_path(
        &map,
        &scene_item.grid_position,
        &grid_point_from_scene_point(&move_to_scene_point, &map),
    ) {
        grid_path.drain(0..1);
        let behavior = match order {
            Order::MoveTo(_) => ItemBehavior::MoveTo(move_to_scene_point, grid_path),
            Order::MoveFastTo(_) => ItemBehavior::MoveFastTo(move_to_scene_point, grid_path),
            Order::HideTo(_) => ItemBehavior::HideTo(move_to_scene_point, grid_path),
        };
        scene_item_modifiers.push(SceneItemModifier::SwitchToNextOrder);
        scene_item_modifiers.push(SceneItemModifier::ChangeBehavior(behavior))
    } else {
        eprintln!("No path found to given scene point {}", move_to_scene_point);
    };

    scene_item_modifiers
}

pub fn digest_move_behavior(
    scene_item: &SceneItem,
    grid_path: &GridPath,
    map: &Map,
) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    if let Some(going_to_grid_point) = grid_path.first() {
        let going_to_scene_point = scene_point_from_grid_point(going_to_grid_point, &map);

        // Note: angle computed by adding FRAC_PI_2 because sprites are north oriented
        scene_item_modifiers.push(SceneItemModifier::ChangeDisplayAngle(angle(
            going_to_scene_point,
            scene_item.position,
        )));

        // Check if scene_point reached
        let distance = going_to_scene_point.distance(scene_item.position);
        let velocity =
            velocity_for_behavior(&scene_item.behavior).expect("must have velocity here");
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

    scene_item_modifiers
}
