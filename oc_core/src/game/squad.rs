use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SquadType {
    // TODO : use real squad types names
    Type1,
    Bren,
    Mg34,
}

impl SquadType {
    pub fn name(&self) -> &str {
        match self {
            SquadType::Type1 => "Type 1",
            SquadType::Bren => "Bren",
            SquadType::Mg34 => "Mg34",
        }
    }
}
