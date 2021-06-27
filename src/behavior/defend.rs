use crate::map::Map;
use crate::scene::item::{SceneItem, SceneItemModifier};
use crate::Angle;

pub fn digest_defend_order(
    _scene_item: &SceneItem,
    _angle: &Angle,
    _map: &Map,
) -> Vec<SceneItemModifier> {
    let scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    scene_item_modifiers
}

pub fn digest_hide_order(
    _scene_item: &SceneItem,
    _angle: &Angle,
    _map: &Map,
) -> Vec<SceneItemModifier> {
    let scene_item_modifiers: Vec<SceneItemModifier> = vec![];

    scene_item_modifiers
}
