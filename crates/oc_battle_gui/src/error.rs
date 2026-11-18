use std::fmt::Display;

use bevy::prelude::*;

#[derive(Debug, Event)]
pub struct Error(pub String);

pub struct ErrorPlugin;

impl Plugin for ErrorPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_error);
    }
}

pub trait OkOrSendError {
    fn ok_or_send(&self, commands: Commands);
}

impl<T, E: Display> OkOrSendError for std::result::Result<T, E> {
    fn ok_or_send(&self, mut commands: Commands) {
        if let Err(error) = self {
            commands.trigger(Error(error.to_string()))
        }
    }
}
pub fn on_error(error: On<Error>) {
    tracing::error!("Error : {}", error.0);
}
