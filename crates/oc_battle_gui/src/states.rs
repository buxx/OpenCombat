use std::time::Instant;

use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    Home,
    Connecting,
    Downloading,
    InGame,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum PointerIn {
    Battle,
    Hud,
    #[default]
    Window,
}

#[derive(Debug, Resource)]
pub struct Game {
    #[allow(unused)]
    pub started: Instant,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

#[derive(Debug, Deref, DerefMut, Resource, Default)]
pub struct GameConfig(pub Option<oc_network::GameConfig>);

#[derive(Deref, DerefMut, Resource, Default)]
pub struct GuiFiles(pub Option<oc_root::files::Files>);

#[derive(Deref, DerefMut, Resource, Default)]
pub struct Window(pub Option<crate::window::Window>);
