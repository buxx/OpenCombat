use crate::types::*;

mod entity;
pub mod local;
mod order;
pub mod shared;
mod squad;

pub enum SideEffect {
    RefreshEntityAnimation(EntityIndex),
}
