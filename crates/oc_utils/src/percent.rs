/// Percentage between 0 and 100 (stored in one byte)
#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(pub u8);

impl Percent {
    pub const ZERO: Percent = Percent(0);
    pub const FULL: Percent = Percent(100);

    pub fn new(value: u8) -> Self {
        Self(value.min(100))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// 0.0 for 0%, 1.0 for 100%
    pub fn ratio(&self) -> f32 {
        self.0 as f32 / 100.
    }

    pub fn from_ratio(ratio: f32) -> Self {
        Self((ratio.clamp(0., 1.) * 100.).round() as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ratio() {
        assert_eq!(Percent(50).ratio(), 0.5);
        assert_eq!(Percent::FULL.ratio(), 1.0);
        assert_eq!(Percent::ZERO.ratio(), 0.0);
    }

    #[test]
    fn test_from_ratio_clamps() {
        assert_eq!(Percent::from_ratio(0.426), Percent(43));
        assert_eq!(Percent::from_ratio(2.0), Percent::FULL);
        assert_eq!(Percent::from_ratio(-1.0), Percent::ZERO);
    }

    #[test]
    fn test_new_clamps() {
        assert_eq!(Percent::new(200), Percent::FULL);
    }
}
