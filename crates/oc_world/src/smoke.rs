use oc_utils::percent::Percent;
use std::time::{Duration, Instant};

use derive_more::Constructor;
use oc_geo::tile::WorldTileIndex;

/// A smoke cloud: tiles keep their opacity during `hold`, then fade to nothing during `fade`.
#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct SmokeCloud {
    pub start: Instant,
    pub hold: Duration,
    pub fade: Duration,
    pub tiles: Vec<(WorldTileIndex, Percent)>,
}

impl SmokeCloud {
    pub fn remaining(&self, now: Instant) -> f32 {
        let elapsed = now.saturating_duration_since(self.start);
        match elapsed.checked_sub(self.hold) {
            None => 1.0,
            Some(fading) if self.fade.is_zero() || fading >= self.fade => 0.0,
            Some(fading) => 1.0 - fading.as_secs_f32() / self.fade.as_secs_f32(),
        }
    }

    pub fn expired(&self, now: Instant) -> bool {
        self.remaining(now) <= 0.0
    }

    pub fn opacities(&self, now: Instant) -> impl Iterator<Item = (WorldTileIndex, Percent)> + '_ {
        let remaining = self.remaining(now);
        self.tiles
            .iter()
            .map(move |(i, opacity)| (*i, Percent::from_ratio(opacity.ratio() * remaining)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cloud(start: Instant) -> SmokeCloud {
        SmokeCloud::new(
            start,
            Duration::from_secs(10),
            Duration::from_secs(10),
            vec![(WorldTileIndex(0), Percent(80))],
        )
    }

    #[test]
    fn test_keeps_opacity_during_hold() {
        let start = Instant::now();
        let at = start + Duration::from_secs(10);
        let opacities: Vec<_> = cloud(start).opacities(at).collect();
        assert_eq!(opacities, vec![(WorldTileIndex(0), Percent(80))]);
    }

    #[test]
    fn test_fades_after_hold() {
        let start = Instant::now();
        let at = start + Duration::from_secs(15);
        let opacities: Vec<_> = cloud(start).opacities(at).collect();
        assert_eq!(opacities, vec![(WorldTileIndex(0), Percent(40))]);
    }

    #[test]
    fn test_expires_after_fade() {
        let start = Instant::now();
        assert!(!cloud(start).expired(start + Duration::from_secs(19)));
        assert!(cloud(start).expired(start + Duration::from_secs(20)));
    }
}
