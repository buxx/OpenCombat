use bevy::prelude::*;

use update::on_update_squad;

use crate::{ingame::squad::menu::contextual, menu::contextual::ContextualMenuPlugin};

#[cfg(feature = "debug")]
pub mod debug;
pub mod menu;
pub mod update;

pub struct SquadPlugin;

impl Plugin for SquadPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ContextualMenuPlugin::<contextual::Menu>::default())
            .add_observer(contextual::on_choose)
            .add_observer(on_update_squad);

        #[cfg(feature = "debug")]
        {
            app.add_plugins(debug::DebugPlugin);
        }
    }
}
