//! Typed indexes: newtypes over an integer, identifying an item by its position in a list.

/// Newtype index (position of an item in a list)
pub trait IndexType: Copy {
    fn from_usize(i: usize) -> Self;
    fn as_usize(&self) -> usize;
}

/// Declare an index newtype usable with [`IndexType`] and [`AtIndex`].
///
/// ```ignore
/// oc_utils::index_type! {
///     /// Index of an individual
///     pub struct IndividualIndex(pub u64);
/// }
/// ```
///
/// Generates the struct (with common derives, including rkyv and serde ones through
/// `oc_derive::archivable`; caller crate must depend on `oc_derive`, `rkyv` and `serde`), plus `new`, [`IndexType`], `Deref<Target = int>`, `Display` and
/// `From<usize>` / `From<int>` implementations.
#[macro_export]
macro_rules! index_type {
    ($(#[$meta:meta])* $vis:vis struct $name:ident(pub $int:ty);) => {
        $(#[$meta])*
        #[oc_derive::archivable(serde)]
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $name(pub $int);

        impl $name {
            pub const fn new(value: $int) -> Self {
                Self(value)
            }
        }

        impl $crate::index::IndexType for $name {
            #[inline]
            fn from_usize(i: usize) -> Self {
                Self(i as $int)
            }

            #[inline]
            fn as_usize(&self) -> usize {
                self.0 as usize
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = $int;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl ::std::convert::From<usize> for $name {
            fn from(value: usize) -> Self {
                <Self as $crate::index::IndexType>::from_usize(value)
            }
        }

        $crate::index_type!(@from_int $name, $int);
    };
    // `From<usize>` is already implemented
    (@from_int $name:ident, usize) => {};
    (@from_int $name:ident, $int:ty) => {
        impl ::std::convert::From<$int> for $name {
            fn from(value: $int) -> Self {
                Self(value)
            }
        }
    };
}

/// Access slice items with a typed index (instead of `items[index.0 as usize]`)
pub trait AtIndex<T> {
    fn at<I: IndexType>(&self, index: I) -> &T;
    fn at_mut<I: IndexType>(&mut self, index: I) -> &mut T;
    fn get_at<I: IndexType>(&self, index: I) -> Option<&T>;
    fn get_at_mut<I: IndexType>(&mut self, index: I) -> Option<&mut T>;
}

impl<T> AtIndex<T> for [T] {
    #[inline]
    fn at<I: IndexType>(&self, index: I) -> &T {
        &self[index.as_usize()]
    }

    #[inline]
    fn at_mut<I: IndexType>(&mut self, index: I) -> &mut T {
        &mut self[index.as_usize()]
    }

    #[inline]
    fn get_at<I: IndexType>(&self, index: I) -> Option<&T> {
        self.get(index.as_usize())
    }

    #[inline]
    fn get_at_mut<I: IndexType>(&mut self, index: I) -> Option<&mut T> {
        self.get_mut(index.as_usize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::index_type! {
        pub struct TestIndex(pub u16);
    }

    #[test]
    fn index_type_conversions() {
        let i = TestIndex::from(3usize);
        assert_eq!(i, TestIndex::new(3));
        assert_eq!(i, TestIndex::from(3u16));
        assert_eq!(*i, 3u16);
        assert_eq!(i.as_usize(), 3);
        assert_eq!(i.to_string(), "3");
    }

    #[test]
    fn at_index() {
        let mut items = vec!['a', 'b', 'c'];
        assert_eq!(items.at(TestIndex(1)), &'b');
        *items.at_mut(TestIndex(2)) = 'z';
        assert_eq!(items, vec!['a', 'b', 'z']);
        assert_eq!(items.get_at(TestIndex(0)), Some(&'a'));
        assert_eq!(items.get_at(TestIndex(3)), None);
    }
}
