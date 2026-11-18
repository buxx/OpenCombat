use std::fmt::Display;

use derive_more::Constructor;

use crate::window::debug::{battle::component::ComponentDetails, physics::PhysicsRepr};

#[derive(Debug, Clone, Constructor)]
pub struct Subject<I: Display> {
    pub i: I,
    pub physics: PhysicsRepr,
    pub infos: Vec<(String, String)>,
    pub details: ComponentDetails,
}

impl<I: Display> Subject<I> {
    pub fn details(&self) -> ComponentDetails {
        self.details
    }
}
