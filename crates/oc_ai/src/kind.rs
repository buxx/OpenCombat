use serde::{Deserialize, Serialize};

use crate::{Commander, Defensive, Offensive};

/// Available AI behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum AiKind {
    /// Hold flags: deploy around them and defend
    #[default]
    #[cfg_attr(feature = "clap", value(alias = "defense"))]
    Defensive,
    /// Take flags: move to them and engage
    Offensive,
}

impl AiKind {
    /// All behaviors
    pub const ALL: [AiKind; 2] = [AiKind::Defensive, AiKind::Offensive];

    /// New commander implementing this behavior
    pub fn commander(self) -> Box<dyn Commander> {
        match self {
            AiKind::Defensive => Box::new(Defensive::default()),
            AiKind::Offensive => Box::new(Offensive),
        }
    }

    /// Command line value (see `--ai-kind`)
    pub fn arg_name(self) -> &'static str {
        match self {
            AiKind::Defensive => "defensive",
            AiKind::Offensive => "offensive",
        }
    }

    /// Name displayed to players
    pub fn label(self) -> &'static str {
        match self {
            AiKind::Defensive => "Defensive",
            AiKind::Offensive => "Offensive",
        }
    }
}
