use derive_more::{Add, Deref};

use crate::physics::Meters;

#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, Deref, PartialEq)]
pub struct Opacity(pub f32);

#[oc_derive::archivable]
#[derive(Debug, Clone, Copy, Deref, PartialEq, PartialOrd, Add)]
pub struct CumulatedOpacity(pub f32);

impl CumulatedOpacity {
    /// Inaccuracy (degrees) caused by this opacity for a fire at `distance`. Bad visibility on a
    /// far target is worse than on a close one: the opacity inaccuracy is scaled by [`distance_factor`].
    pub fn inaccuracy(&self, w: &crate::WorldConfig, distance: Meters) -> f32 {
        self.0 * w.opacity_inaccuracy * distance_factor(w, distance)
    }
}

/// `distance / reference distance`, bounded by configured min and max factors
pub fn distance_factor(w: &crate::WorldConfig, distance: Meters) -> f32 {
    let reference = w.opacity_inaccuracy_reference_distance.0;
    let (min, max) = (
        w.opacity_inaccuracy_min_distance_factor,
        w.opacity_inaccuracy_max_distance_factor,
    );
    if reference <= 0. {
        return max;
    }
    (distance.0 / reference).clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorldConfig;

    fn w() -> WorldConfig {
        WorldConfig::new(100, 100)
            .with_opacity_inaccuracy(2.)
            .with_opacity_inaccuracy_reference_distance(Meters(100.))
            .with_opacity_inaccuracy_min_distance_factor(0.25)
            .with_opacity_inaccuracy_max_distance_factor(3.)
    }

    #[test]
    fn reference_distance_keeps_opacity_inaccuracy() {
        let w = w();
        let inaccuracy = CumulatedOpacity(0.5).inaccuracy(&w, Meters(100.));
        assert!((inaccuracy - 1.).abs() < 1e-6);
    }

    #[test]
    fn farther_is_more_inaccurate() {
        let w = w();
        let near = CumulatedOpacity(0.5).inaccuracy(&w, Meters(100.));
        let far = CumulatedOpacity(0.5).inaccuracy(&w, Meters(200.));
        assert!((far - 2. * near).abs() < 1e-6);
    }

    #[test]
    fn no_opacity_no_distance_effect() {
        let w = w();
        assert_eq!(CumulatedOpacity(0.).inaccuracy(&w, Meters(1000.)), 0.);
    }

    #[test]
    fn distance_factor_is_bounded() {
        let w = w();
        assert!((distance_factor(&w, Meters(1.)) - 0.25).abs() < 1e-6);
        assert!((distance_factor(&w, Meters(10_000.)) - 3.).abs() < 1e-6);
    }
}
