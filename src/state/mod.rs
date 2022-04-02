use crate::types::*;

pub mod local;
mod order;
pub mod shared;
mod soldier;
mod squad;
mod vehicle;

pub enum SideEffect {
    RefreshEntityAnimation(SoldierIndex),
}
