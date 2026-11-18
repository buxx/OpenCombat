use bevy::prelude::*;
use oc_root::{
    WcfgFrom,
    geo::{ScreenVec2, WorldVec2},
};
use oc_utils::{let_ok, let_some};
use strum::IntoEnumIterator;

use crate::{
    menu::contextual::{
        ContextMenu, ContextualMenu,
        choice::{Choice, DisabledChoice, choose, context_item},
    },
    states::GameConfig,
};

pub trait OpenContextualMenu<T: ContextualMenu + Send + Sync + 'static> {
    fn position(&self) -> WorldVec2;
    fn is_disabled(&self, _choice: &T::Choices) -> bool {
        false
    }
}

pub fn on_open<T>(
    event: On<T::OpenEvent>,
    g: Res<GameConfig>,
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
) where
    T: ContextualMenu + Send + Sync + 'static,
    for<'a> <<T as ContextualMenu>::ChoiceEvent as Event>::Trigger<'a>: std::default::Default,
{
    let_some!(g = &g.0, return);
    let (camera, camera_transform) = *camera;

    let position = event.position();
    let position = ScreenVec2::from_(position, &g.w);
    let position = camera.world_to_viewport(camera_transform, position.extend(0.).into());
    let_ok!(position = position, return);

    commands
        .spawn((
            Name::new("context menu"),
            ContextMenu::<T>::default(),
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                top: px(position.y),
                flex_direction: FlexDirection::Column,
                // border_radius: BorderRadius::all(px(4)),
                ..default()
            },
        ))
        .with_children(|parent| {
            T::Choices::iter().for_each(|item| {
                let disabled = event.is_disabled(&item);
                let rect = if disabled {
                    item.disabled()
                } else {
                    item.idle()
                };
                let image = ImageNode::new(asset_server.load(T::image())).with_rect(rect);
                parent
                    .spawn(context_item(item, image))
                    .insert_if(DisabledChoice, || disabled);
            })
        })
        .observe(choose::<T>);
}
