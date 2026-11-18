use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Weapons {
    pub main: Option<String>,
    #[serde(default)]
    pub grenades: Vec<String>,
}

impl Weapons {
    pub fn empty() -> Weapons {
        Weapons {
            main: None,
            grenades: vec![],
        }
    }
}
