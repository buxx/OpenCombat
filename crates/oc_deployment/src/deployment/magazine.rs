use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Magazine {
    pub name: String,
    // FIXME BS NOW: add possibility to precise magazine and cartridge in it
    // (then update crates/oc_deployment/src/deployment/mod.rs)
}
