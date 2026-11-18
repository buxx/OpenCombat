use derive_more::Constructor;

#[derive(Debug, Clone, Constructor)]
pub struct Place {
    pub name: PlaceName,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct PlaceName(pub String);

impl From<&str> for PlaceName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for PlaceName {
    fn from(value: String) -> Self {
        Self(value)
    }
}
