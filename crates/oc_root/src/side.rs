#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum,))]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    A,
    B,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }

    pub fn letter(&self) -> char {
        match self {
            Side::A => 'A',
            Side::B => 'B',
        }
    }
}
