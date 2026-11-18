use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub squads: Vec<Squad>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Squad {
    #[serde(default = "default_count")]
    pub count: usize,
    pub label: String,
    pub illustration: String,
    pub individuals: Vec<Individual>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Individual {
    #[serde(default = "default_count")]
    pub count: usize,
    pub illustration: String,
    /// `[base, range]`: individual xp is `base` plus or minus a random value up to `range`
    #[serde(default = "default_xp")]
    pub xp: [u8; 2],
    pub weapons: Weapons,
    pub magazines: Vec<Magazine>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weapons {
    pub main: Option<String>,
    #[serde(default)]
    pub grenades: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Magazine {
    #[serde(default = "default_count")]
    pub count: usize,
    pub name: String,
}

fn default_count() -> usize {
    1
}

fn default_xp() -> [u8; 2] {
    [128, 75]
}
