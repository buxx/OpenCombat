use std::marker::PhantomData;

use bevy::prelude::*;

use crate::menu::contextual::{ContextMenu, ContextualMenu};

#[derive(Event)]
pub struct CloseContextMenu<T: ContextualMenu + Send + Sync + 'static>(PhantomData<T>);

impl<T: ContextualMenu + Send + Sync + 'static> Default for CloseContextMenu<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub fn on_close<T: ContextualMenu + Send + Sync + 'static>(
    _event: On<CloseContextMenu<T>>,
    mut commands: Commands,
    menus: Query<Entity, With<ContextMenu<T>>>,
) {
    for e in menus.iter() {
        commands.entity(e).despawn();
    }
}
