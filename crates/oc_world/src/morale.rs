#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Morale {
    pub a: f32,
    pub b: f32,
}

impl Default for Morale {
    fn default() -> Self {
        Self { a: 100., b: 100. }
    }
}
