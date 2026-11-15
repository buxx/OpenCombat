//! Generic mechanism shared by all mod items: an item is loaded from a `.ron` file, then stored
//! with its position as index (`Indexed<SomeIndex, Item>`).

use std::{
    hash::{Hash, Hasher},
    ops::Deref,
    path::Path,
};

use oc_root::format::FileError;
use rkyv::Archive;
use serde::de::DeserializeOwned;

/// Archivable type whose archived form is `Debug` (required by `derive(Debug)` on archived generics)
pub trait ArchivedDebug: Archive<Archived: std::fmt::Debug> {}

impl<T: Archive<Archived: std::fmt::Debug>> ArchivedDebug for T {}

pub use oc_utils::index::IndexType;

/// A mod item with its index. Derefs to the item.
#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indexed<I: ArchivedDebug, T: ArchivedDebug>(pub I, pub T);

impl<I: ArchivedDebug + Copy, T: ArchivedDebug> Indexed<I, T> {
    pub fn index(&self) -> I {
        self.0
    }

    pub fn inner(&self) -> &T {
        &self.1
    }
}

impl<I: ArchivedDebug, T: ArchivedDebug> Deref for Indexed<I, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

// Items are identified by their index
impl<I: ArchivedDebug + Hash, T: ArchivedDebug> Hash for Indexed<I, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Give to each item the index of its position
pub fn index_all<I: IndexType + ArchivedDebug, T: ArchivedDebug>(
    items: impl IntoIterator<Item = T>,
) -> Vec<Indexed<I, T>> {
    items
        .into_iter()
        .enumerate()
        .map(|(i, item)| Indexed(I::from_usize(i), item))
        .collect()
}

/// Read and parse the `.ron` file named `file` in `path` directory
pub fn read_ron<T, E>(path: &Path, file: &str) -> Result<T, E>
where
    T: DeserializeOwned,
    E: From<FileError>,
{
    Ok(oc_root::format::read_ron(&path.join(file))?)
}
