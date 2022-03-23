use serde::{Deserialize, Serialize};

pub mod squad;

#[derive(Debug, Copy, Serialize, Deserialize, Clone, PartialEq)]
pub enum Side {
    A,
    B,
}
