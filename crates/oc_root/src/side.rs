#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum,))]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    A,
    B,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }

    pub fn letter(&self) -> char {
        match self {
            Side::A => 'A',
            Side::B => 'B',
        }
    }
}

impl Side {
    pub const ALL: [Side; 2] = [Side::A, Side::B];
}

/// One value for each side. Indexable by [`Side`].
///
/// Serialized (serde) with `side_a` and `side_b` field names. Its archived (rkyv) layout is the
/// same as two consecutive fields of type `T`.
#[oc_derive::archivable(serde, bare)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BySide<T> {
    #[serde(rename = "side_a")]
    pub a: T,
    #[serde(rename = "side_b")]
    pub b: T,
}

impl<T> BySide<T> {
    pub const fn new(a: T, b: T) -> Self {
        Self { a, b }
    }

    pub fn from_fn(mut f: impl FnMut(Side) -> T) -> Self {
        Self::new(f(Side::A), f(Side::B))
    }

    pub fn get(&self, side: Side) -> &T {
        match side {
            Side::A => &self.a,
            Side::B => &self.b,
        }
    }

    pub fn get_mut(&mut self, side: Side) -> &mut T {
        match side {
            Side::A => &mut self.a,
            Side::B => &mut self.b,
        }
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> BySide<U> {
        BySide::new(f(self.a), f(self.b))
    }

    pub fn as_ref(&self) -> BySide<&T> {
        BySide::new(&self.a, &self.b)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Side, &T)> {
        [(Side::A, &self.a), (Side::B, &self.b)].into_iter()
    }
}

impl<T: rkyv::Archive<Archived: std::fmt::Debug>> std::fmt::Debug for ArchivedBySide<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchivedBySide")
            .field("a", &self.a)
            .field("b", &self.b)
            .finish()
    }
}

impl<T> IntoIterator for BySide<T> {
    type Item = (Side, T);
    type IntoIter = std::array::IntoIter<(Side, T), 2>;

    fn into_iter(self) -> Self::IntoIter {
        [(Side::A, self.a), (Side::B, self.b)].into_iter()
    }
}

impl<T> std::ops::Index<Side> for BySide<T> {
    type Output = T;

    fn index(&self, side: Side) -> &T {
        self.get(side)
    }
}

impl<T> std::ops::IndexMut<Side> for BySide<T> {
    fn index_mut(&mut self, side: Side) -> &mut T {
        self.get_mut(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn by_side_access() {
        let mut values = BySide::from_fn(|side| side.letter());
        assert_eq!(values[Side::A], 'A');
        assert_eq!(values[Side::B], 'B');
        values[Side::B] = 'Z';
        assert_eq!(values.get(Side::B), &'Z');
        assert_eq!(
            values.iter().collect::<Vec<_>>(),
            vec![(Side::A, &'A'), (Side::B, &'Z')]
        );
        assert_eq!(values.map(|c| c.is_lowercase()), BySide::new(false, false));
        assert_eq!(
            values.into_iter().collect::<Vec<_>>(),
            vec![(Side::A, 'A'), (Side::B, 'Z')]
        );
    }

    #[test]
    fn by_side_keeps_formats() {
        #[derive(rkyv::Archive, rkyv::Serialize, serde::Serialize)]
        struct Before {
            before: u8,
            side_a: Vec<String>,
            side_b: Vec<String>,
            after: u32,
        }
        #[derive(rkyv::Archive, rkyv::Serialize, serde::Serialize)]
        struct After {
            before: u8,
            #[serde(flatten)]
            sides: BySide<Vec<String>>,
            after: u32,
        }
        let a = vec!["x".to_string(), "y".to_string()];
        let b = vec!["z".to_string()];
        let before = Before {
            before: 1,
            side_a: a.clone(),
            side_b: b.clone(),
            after: 2,
        };
        let after = After {
            before: 1,
            sides: BySide::new(a, b),
            after: 2,
        };

        assert_eq!(
            rkyv::to_bytes::<rkyv::rancor::Error>(&before)
                .unwrap()
                .as_slice(),
            rkyv::to_bytes::<rkyv::rancor::Error>(&after)
                .unwrap()
                .as_slice(),
        );
        assert_eq!(
            serde_json::to_string(&before).unwrap(),
            serde_json::to_string(&after).unwrap()
        );
    }
}
