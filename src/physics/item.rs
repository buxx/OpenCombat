use crate::behavior::ItemBehavior;
use crate::map::Map;
use crate::physics::util::{grid_point_from_scene_point, scene_point_from_grid_point};
use crate::scene::item::{SceneItem, SceneItemModifier};
use crate::scene::main::MainStateModifier;
use crate::util::velocity_for_behavior;
use crate::{Message, ScenePoint};

pub fn produce_physics_messages_for_scene_item(
    scene_item_i: usize,
    scene_item: &SceneItem,
    map: &Map,
) -> Vec<Message> {
    let mut messages: Vec<Message> = vec![];

    match &scene_item.state.current_behavior {
        ItemBehavior::Standing => {}
        ItemBehavior::MoveTo(move_to_scene_point, grid_path)
        | ItemBehavior::MoveFastTo(move_to_scene_point, grid_path)
        | ItemBehavior::HideTo(move_to_scene_point, grid_path) => {
            if let Some(going_to_grid_point) = grid_path.first() {
                let going_to_scene_point = scene_point_from_grid_point(going_to_grid_point, &map);

                let velocity = velocity_for_behavior(&scene_item.state.current_behavior)
                    .expect("must have velocity here");
                let move_vector =
                    (going_to_scene_point - scene_item.position).normalize() * velocity;
                let new_position = ScenePoint::new(
                    scene_item.position.x + move_vector.x,
                    scene_item.position.y + move_vector.y,
                );
                messages.push(Message::SceneItemMessage(
                    scene_item_i,
                    SceneItemModifier::ChangePosition(new_position),
                ));
                let new_grid_position = grid_point_from_scene_point(&scene_item.position, &map);
                if scene_item.grid_position != new_grid_position {
                    messages.push(Message::MainStateMessage(
                        MainStateModifier::ChangeSceneItemGridPosition(
                            scene_item_i,
                            scene_item.grid_position.clone(),
                            new_grid_position.clone(),
                        ),
                    ));
                    messages.push(Message::SceneItemMessage(
                        scene_item_i,
                        SceneItemModifier::ChangeGridPosition(new_grid_position.clone()),
                    ));
                }
            }
        }
    }

    messages
}
