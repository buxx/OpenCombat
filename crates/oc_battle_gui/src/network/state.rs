use bevy::prelude::*;
use oc_root::identity::Identity;

use crate::config::Connect;

#[derive(Debug, Default, Resource)]
pub struct State {
    pub server: Option<Connect>,
    pub identity: Option<Identity>,
    pub connected: bool,
}
