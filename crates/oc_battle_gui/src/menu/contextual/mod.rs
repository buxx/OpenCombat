use std::marker::PhantomData;

use bevy::prelude::*;

use crate::menu::contextual::open::OpenContextualMenu;

pub mod choice;
pub mod close;
pub mod open;

#[derive(Debug)]
pub struct ContextualMenuPlugin<T: Send + Sync + 'static> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: ContextualMenu + Send + Sync + 'static> Default for ContextualMenuPlugin<T> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<T> Plugin for ContextualMenuPlugin<T>
where
    T: ContextualMenu + Send + Sync + 'static,
    for<'a> <T::ChoiceEvent as Event>::Trigger<'a>: Default,
{
    fn build(&self, app: &mut App) {
        app
            // On open menu event
            .add_observer(open::on_open::<T>)
            // On hover menu
            .add_observer(choice::on_over::<T>)
            // On over menu
            .add_observer(choice::on_out::<T>)
            // On close menu
            .add_observer(close::on_close::<T>);
    }
}

#[derive(Component)]
pub struct ContextMenu<T: ContextualMenu>(PhantomData<T>);

impl<T: ContextualMenu> Default for ContextMenu<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub trait ContextualMenu: std::fmt::Debug + Send + Sync + 'static {
    /// Event type would must be triggered to open the menu
    type OpenEvent: Event + OpenContextualMenu<Self>
    where
        Self: Sized;
    /// Event type which will be triggered on choice click
    type ChoiceEvent: Event + Clone + std::fmt::Debug;
    /// Type which represent menu choices
    type Choices: choice::Choice<Self::ChoiceEvent>;

    fn image() -> &'static str;
}
