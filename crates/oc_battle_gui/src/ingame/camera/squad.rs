#[cfg(feature = "debug")]
use bevy::prelude::*;

#[cfg(feature = "debug")]
#[derive(Debug, Event)]
pub struct ToggleShowFormationPositions;

#[cfg(feature = "debug")]
#[derive(Debug, Resource, Deref, DerefMut, Default)]
pub struct ShowFormationPositions(pub bool);

#[cfg(feature = "debug")]
pub fn on_toggle_show_formation_positions(
    _: On<ToggleShowFormationPositions>,
    mut show: ResMut<ShowFormationPositions>,
) {
    show.0 = !show.0;
}
