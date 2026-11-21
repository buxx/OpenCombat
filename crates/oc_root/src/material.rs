#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaterialKind {
    Individual,
    Vehicle,
    Projectile,
}
