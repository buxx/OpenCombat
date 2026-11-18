use derive_more::Deref;
use std::time::{Duration, Instant};

use oc_root::{
    geo::WorldVec2,
    side::{BySide, Side},
};

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SupportType(pub String);

impl SupportType {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SupportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Default, PartialEq, Deref)]
/// Count of available supports by type
pub struct Support(pub Vec<(SupportType, u8)>);

/// Supports of each side
pub type Supports = BySide<Support>;

impl Support {
    pub fn count(&self, type_: &SupportType) -> u8 {
        self.0
            .iter()
            .find(|(t, _)| t == type_)
            .map(|(_, count)| *count)
            .unwrap_or_default()
    }

    pub fn consume(&mut self, type_: &SupportType) {
        if let Some((_, count)) = self.0.iter_mut().find(|(t, _)| t == type_) {
            *count = count.saturating_sub(1);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSupport {
    pub type_: SupportType,
    pub side: Side,
    pub position: WorldVec2,
    pub start: Option<Instant>,
    pub delay: Duration,
}

impl PendingSupport {
    pub fn fresh(side: Side, type_: SupportType, position: WorldVec2, delay: Duration) -> Self {
        Self {
            type_,
            side,
            position,
            start: None,
            delay,
        }
    }

    /// Same side, type and position (pending of same kind can coexist)
    pub fn same_request(&self, other: &PendingSupport) -> bool {
        self.side == other.side && self.type_ == other.type_ && self.position == other.position
    }

    pub fn type_(&self) -> &SupportType {
        &self.type_
    }

    pub fn side(&self) -> Side {
        self.side
    }

    /// Preparation time starts only when the fight begins
    pub fn is_started(&self) -> bool {
        self.start.is_some()
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now() + self.delay);
    }

    /// `support` is the mod declaration of this pending support type
    pub fn is_ready(&self, support: &oc_mod::support::Support, speed: f32) -> bool {
        let prepare = support.prepare().div_f32(speed);
        self.start
            .map(|s| s.elapsed() >= prepare)
            .unwrap_or_default()
    }
}
