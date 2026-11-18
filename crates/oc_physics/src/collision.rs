#[cfg(feature = "bevy")]
use bevy::prelude::*;
use oc_root::material::MaterialKind;

pub trait Material {
    fn kind(&self) -> Option<MaterialKind>;
}

#[cfg(feature = "bevy")]
#[derive(Debug, Deref, DerefMut, Component)]
pub struct Material_(pub Option<MaterialKind>);
