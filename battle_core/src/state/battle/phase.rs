use serde::{Deserialize, Serialize};
use strum_macros::Display;
use strum_macros::EnumIter;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Display, EnumIter)]
pub enum Phase {
    Placement,
    Battle,
    End,
}

impl Phase {
    pub fn placement(&self) -> bool {
        match self {
            Phase::Placement => true,
            _ => false,
        }
    }

    pub fn battle(&self) -> bool {
        match self {
            Phase::Battle => true,
            _ => false,
        }
    }

    pub fn end(&self) -> bool {
        match self {
            Phase::End => true,
            _ => false,
        }
    }
}
