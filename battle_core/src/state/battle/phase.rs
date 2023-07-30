use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

use crate::game::Side;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Display, EnumIter)]
pub enum Phase {
    Placement,
    Battle,
    End(Victorious, EndReason),
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
            Phase::End(_, _) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Display)]
pub enum EndReason {
    Flags,
    Morale,
}

// To be compliant with Phase EnumIter
impl Default for EndReason {
    fn default() -> Self {
        Self::Flags
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Victorious(pub Side);
impl Victorious {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

// To be compliant with Phase EnumIter
impl Default for Victorious {
    fn default() -> Self {
        Self(Side::A)
    }
}
