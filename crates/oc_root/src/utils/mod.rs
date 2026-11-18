use std::time::Duration;

pub fn remove_numeric_suffix(s: &str) -> &str {
    // Find last '_' followed by only digits until end of string
    if let Some(pos) = s.rfind('_')
        && s[pos + 1..].chars().all(|c| c.is_ascii_digit())
        && !s[pos + 1..].is_empty()
    {
        return &s[..pos];
    }
    s
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct U8Progress(pub u8);

impl U8Progress {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn tick(&self, increment: u8) -> (Self, Self) {
        let add = increment.min(255 - self.0);
        let new = self.0 + add;
        let exceedance = increment - add;
        (Self(new), Self(exceedance))
    }

    pub fn finished(&self) -> bool {
        self.0 == 255
    }
    pub fn f32(&self) -> f32 {
        self.0 as f32 / 255.
    }
}

#[oc_derive::archivable(serde)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frequency {
    interval: Duration,
    hz: f32,
    period: f32,
}

impl Frequency {
    pub fn new(hz: f32) -> Self {
        let interval = Duration::from_micros((1_000_000. / hz) as u64);
        let period = 1. / hz;
        Self {
            interval,
            hz,
            period,
        }
    }

    pub fn each(interval: Duration) -> Self {
        let hz = 1_000_000. / interval.as_micros() as f32;
        let period = 1. / hz;
        Self {
            interval,
            hz,
            period,
        }
    }

    pub fn interval(&self) -> Duration {
        self.interval
    }

    pub fn hz(&self) -> f32 {
        self.hz
    }

    pub fn period(&self) -> f32 {
        self.period
    }
}

impl std::ops::Mul<f32> for Frequency {
    type Output = Frequency;

    fn mul(self, rhs: f32) -> Frequency {
        Frequency::new(self.hz * rhs)
    }
}

// WARN: U8Progress tests AI generated
#[cfg(test)]
mod tests {
    use super::*;

    // AI generated test
    #[test]
    fn test_from_zero() {
        let progress = U8Progress::zero();
        assert_eq!(progress.0, 0);
        assert!(!progress.finished());

        let (new, exceedance) = progress.tick(50);
        assert_eq!(new.0, 50);
        assert_eq!(exceedance.0, 0);
        assert!(!new.finished());
    }

    // AI generated test
    #[test]
    fn test_reach_255_exactly() {
        let progress = U8Progress(200);
        let (new, exceedance) = progress.tick(55);
        assert_eq!(new.0, 255);
        assert_eq!(exceedance.0, 0);
        assert!(new.finished());
    }

    // AI generated test
    #[test]
    fn test_tick_with_exceedance() {
        let progress = U8Progress(200);
        let (new, exceedance) = progress.tick(100);
        assert_eq!(new.0, 255);
        assert_eq!(exceedance.0, 45);
        assert!(new.finished());
    }

    #[test]
    fn test_remove_numeric_suffix() {
        assert_eq!(remove_numeric_suffix("toto_1"), "toto");
        assert_eq!(remove_numeric_suffix("toto_a_2"), "toto_a");
        assert_eq!(remove_numeric_suffix("foo_bar_42"), "foo_bar");
        assert_eq!(remove_numeric_suffix("no_suffix"), "no_suffix");
        assert_eq!(remove_numeric_suffix("trailing_"), "trailing_");
        assert_eq!(remove_numeric_suffix("abc_10_def"), "abc_10_def");
    }
}
