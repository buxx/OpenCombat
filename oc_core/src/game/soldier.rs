use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SoldierType {
    // TODO : use real soldier types names
    Type1,
    Bren,
    Mg34,
}

impl SoldierType {
    pub fn name(&self) -> &str {
        match self {
            SoldierType::Type1 => "Type 1",
            SoldierType::Bren => "Bren",
            SoldierType::Mg34 => "Mg34",
        }
    }
}
