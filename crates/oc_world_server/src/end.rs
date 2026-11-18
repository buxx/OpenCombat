use std::{path::PathBuf, str::FromStr};

/// What the server does with the world when the battle ends
#[derive(Debug, Clone, Default, PartialEq)]
pub enum EndPolicy {
    #[default]
    None,
    SaveMapPersistTo(PathBuf),
}

const NONE: &str = "none";
const SAVE_MAP_PERSIST_TO: &str = "save-map-persist-to:";

impl FromStr for EndPolicy {
    type Err = String;

    /// Accepted values: `none`, `save-map-persist-to:<path>`
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == NONE {
            return Ok(Self::None);
        }

        if let Some(path) = value.strip_prefix(SAVE_MAP_PERSIST_TO) {
            if path.is_empty() {
                return Err(format!("Missing path after '{SAVE_MAP_PERSIST_TO}'"));
            }
            return Ok(Self::SaveMapPersistTo(PathBuf::from(path)));
        }

        Err(format!(
            "Unknown end policy '{value}' (expected '{NONE}' or '{SAVE_MAP_PERSIST_TO}<path>')"
        ))
    }
}
