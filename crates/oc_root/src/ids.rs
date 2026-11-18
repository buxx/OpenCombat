use std::{
    ops::Deref,
    sync::{Arc, atomic::AtomicU64},
};

#[derive(Debug, Clone, Default)]
pub struct Ids {
    pub projectiles: Projectiles,
}

#[derive(Debug, Clone, Default)]
pub struct Projectiles(Arc<AtomicU64>);

impl Deref for Projectiles {
    type Target = AtomicU64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
