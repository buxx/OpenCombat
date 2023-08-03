pub enum SquadType {
    // TODO : use real squad types names
    Type1,
}

impl SquadType {
    pub fn name(&self) -> &str {
        match self {
            SquadType::Type1 => "Type 1",
        }
    }
}
