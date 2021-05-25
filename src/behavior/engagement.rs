use crate::map::Map;
use crate::scene::item::{SceneItem, SceneItemModifier};
use crate::SceneItemId;
use rand::Rng;

const DEFAULT_FRAMES_TO_ACQUIRE: u32 = 120;

pub fn digest_engage_scene_item_behavior(
    frame_i: u32,
    scene_item: &SceneItem,
    engage_scene_item_id: SceneItemId,
    _map: &Map,
) -> Vec<SceneItemModifier> {
    let mut scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    // FIXME BS NOW: il faut reussir a Disengage lorsque la target est morte ...
    if let Some(visibility) = scene_item.visible_scene_items_visibilities_for(engage_scene_item_id)
    {
        // Always acquire a target before fire
        if let Some(acquiring_until) = scene_item.acquiring_until {
            if acquiring_until <= frame_i {
                scene_item_modifiers.push(SceneItemModifier::FireOnSceneItem(visibility.clone()))
            }
        } else {
            let mut rng = rand::thread_rng(); // TODO: Does it cost resources ?
                                              // TODO: determine with visibility, stress, etc ...
            let until_frame_i =
                frame_i + DEFAULT_FRAMES_TO_ACQUIRE + rng.gen_range(0..DEFAULT_FRAMES_TO_ACQUIRE);
            scene_item_modifiers.push(SceneItemModifier::AcquireUntil(until_frame_i));
        }
    } else {
        scene_item_modifiers.push(SceneItemModifier::Disengage);
    }

    scene_item_modifiers
}
