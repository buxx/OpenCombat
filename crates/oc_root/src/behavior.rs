#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Suppress(pub u8);

impl Suppress {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn normalize(&self) -> f32 {
        self.0 as f32 / 255.0
    }

    pub fn decrease(mut self, value: Suppress) -> Suppress {
        self.0 = self.0.saturating_sub(value.0);
        self
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stun(pub u8);

impl Stun {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn decrease(mut self, value: Stun) -> Stun {
        self.0 = self.0.saturating_sub(value.0);
        self
    }
}
