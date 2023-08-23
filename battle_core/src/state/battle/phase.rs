use std::fmt::Display;

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
    pub fn is_placement(&self) -> bool {
        matches!(self, Phase::Placement)
    }

    pub fn is_battle(&self) -> bool {
        matches!(self, Phase::Battle)
    }

    pub fn is_end(&self) -> bool {
        matches!(self, Phase::End(_, _))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Display)]
pub enum EndReason {
    Flags,
    Morale,
    Aborted,
}

// To be compliant with Phase EnumIter
impl Default for EndReason {
    fn default() -> Self {
        Self::Flags
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Victorious(pub Side);

impl Display for Victorious {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Side::All => f.write_str("Undetermined"),
            Side::A | Side::B => f.write_str(&self.0.to_string()),
        }
    }
}

// To be compliant with Phase EnumIter
impl Default for Victorious {
    fn default() -> Self {
        Self(Side::A)
    }
}
