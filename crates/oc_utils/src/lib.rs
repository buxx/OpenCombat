// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

#[cfg(feature = "bevy")]
pub mod bevy;
pub mod collections;
pub mod d2;
#[cfg(feature = "debug")]
pub mod debug;
pub mod error;
pub mod image;
pub mod index;
pub mod number;
pub mod percent;
pub mod polygon;
pub mod random;
#[cfg(feature = "tiled")]
pub mod tileset;

#[macro_export]
macro_rules! let_some {
    ($pat:pat = $expr:expr, $or:expr) => {
        let Some($pat) = $expr else { $or };
    };
}

#[macro_export]
macro_rules! let_ok {
    ($pat:pat = $expr:expr, $or:expr) => {
        let Ok($pat) = $expr else { $or };
    };
}

#[macro_export]
macro_rules! true_ {
    ($expr:expr, $or:expr) => {
        if !$expr {
            $or
        };
    };
}

#[macro_export]
macro_rules! if_true {
    ($expr:expr, $then:expr) => {
        if $expr {
            $then
        };
    };
}

#[macro_export]
macro_rules! false_ {
    ($expr:expr, $or:expr) => {
        if $expr {
            $or
        };
    };
}

#[macro_export]
macro_rules! return_if {
    ($expr:expr) => {
        if $expr {
            return;
        }
    };
    ($expr:expr, $return: expr) => {
        if $expr {
            $return;
        }
    };
}
