use anyhow::{Context, bail};
use oc_mod::{Mod, ammunition::IndexedAmmunition, magazine::IndexedMagazine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Magazine {
    pub name: String,
    /// Ammunition name filling this magazine. When `None`, the first ammunition
    /// compatible with the magazine (and the weapon, if inserted in it) is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ammunition: Option<String>,
}

impl Magazine {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ammunition: None,
        }
    }

    pub fn with_ammunition(mut self, ammunition: String) -> Self {
        self.ammunition = Some(ammunition);
        self
    }

    /// Resolve the precised ammunition from mod and check it is accepted by `magazine`.
    /// Return `None` if no ammunition is precised.
    pub fn ammunition<'a>(
        &self,
        mod_: &'a Mod,
        magazine: &IndexedMagazine,
    ) -> Result<Option<&'a IndexedAmmunition>, anyhow::Error> {
        let Some(name) = &self.ammunition else {
            return Ok(None);
        };

        let ammunition = mod_
            .ammunition_from_name(name)
            .context(format!("Use ammunition '{name}'"))?;
        if !magazine.accept().contains(ammunition.model()) {
            bail!(
                "Ammunition '{name}' (model '{}') is not accepted by magazine '{}'",
                ammunition.model(),
                self.name
            );
        }

        Ok(Some(ammunition))
    }
}
