use std::hash::Hash;

use bevy::prelude::*;
use rustc_hash::FxHashMap;

#[derive(Debug, Resource, Deref, DerefMut)]
pub struct EntityMapping<I: Hash>(pub FxHashMap<I, Entity>);

impl<I: Hash> Default for EntityMapping<I> {
    fn default() -> Self {
        Self(FxHashMap::default())
    }
}

#[macro_export]
macro_rules! every {
    ($timer:ident, $time:expr, $period:expr) => {
        let $timer = $timer.get_or_insert_with(|| Timer::new($period, TimerMode::Repeating));
        if !$timer.tick($time.delta()).just_finished() {
            return;
        }
    };
}
