use serde::{Deserialize, Serialize};

pub mod control;
pub mod cover;
pub mod explosive;
pub mod flag;
pub mod health;
pub mod posture;
pub mod squad;
pub mod weapon;

#[derive(Debug, Copy, Serialize, Deserialize, Clone, PartialEq)]
pub enum Side {
    All,
    A,
    B,
}
impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::All => Side::All,
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }
}

use std::{fmt::Display, str::FromStr};

// any error type implementing Display is acceptable.
type ParseError = &'static str;

impl FromStr for Side {
    type Err = ParseError;
    fn from_str(day: &str) -> Result<Self, Self::Err> {
        match day {
            "a" => Ok(Side::A),
            "b" => Ok(Side::B),
            _ => Err("Could not parse a side"),
        }
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::All => f.write_str("All"),
            Side::A => f.write_str("A"),
            Side::B => f.write_str("B"),
        }
    }
}
