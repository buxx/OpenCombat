#[oc_derive::archivable]
#[derive(Debug, Clone, PartialEq)]
pub enum StaticSource {
    /// Files must be downloaded from the http static server of this port
    Remote(u16),
    /// Server and client share files
    Local,
}
