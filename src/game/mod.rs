use serde::{Deserialize, Serialize};

pub mod squad;

#[derive(Debug, Copy, Serialize, Deserialize, Clone, PartialEq)]
pub enum Side {
    A,
    B,
}

use std::str::FromStr;

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
