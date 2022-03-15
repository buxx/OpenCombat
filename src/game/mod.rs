use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Serialize, Deserialize, Clone, PartialEq)]
pub enum Side {
    A,
    B,
}
