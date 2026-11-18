// derive_more::Constructor triggers a false positive
#![allow(clippy::redundant_field_names)]

use derive_more::Constructor;
use line_drawing::Bresenham3d;
use oc_geo::tile::TileXy;
use oc_mod::nature::Nature;
use oc_root::{
    WcfgFrom, WorldConfig,
    geo::WorldVec3,
    opacity::{CumulatedOpacity, Opacity},
};
use oc_utils::d2::Xy;
use oc_utils::percent::Percent;

#[derive(Debug, Constructor)]
pub struct PathBuilder<'a, F, I>
where
    F: Fn(Xy, f32) -> I,
    I: IntoIterator<Item = Step>,
{
    w: &'a WorldConfig,
    at: F,
}

impl<'a, F, I> PathBuilder<'a, F, I>
where
    F: Fn(Xy, f32) -> I,
    I: IntoIterator<Item = Step>,
{
    pub fn build(
        &self,
        start: WorldVec3,
        end: WorldVec3,
        ignore: usize,
        modifier: Modifier,
    ) -> Path {
        tracing::trace!(name="lov-path-build", start=?start, end=?end, ignore=ignore, modifier=?modifier);
        let mut opacity = CumulatedOpacity(0.);
        let mut tile = TileXy::from_([start.x, start.y], self.w);
        let mut sections = vec![];
        let mut last = start;
        let mut ignored = 0usize;
        let ignore_end_opacity = matches!(modifier, Modifier::None(true));

        // Ceil to prevent > 0 been cast to 0
        let start_ = (
            start.x.ceil() as isize,
            start.y.ceil() as isize,
            start.z.ceil() as isize,
        );
        // Ceil to prevent > 0 been cast to 0
        let end_ = (
            end.x.ceil() as isize,
            end.y.ceil() as isize,
            end.z.ceil() as isize,
        );
        let last_xy = Xy::from_((end_.0, end_.1), self.w);

        tracing::trace!(name="lov-path-build-bresenham3d", start_=?start_, end_=?end_);
        for (pixel_x, pixel_y, pixel_z) in Bresenham3d::new(start_, end_) {
            let pixel: WorldVec3 = [pixel_x as f32, pixel_y as f32, pixel_z as f32].into();
            let xy = Xy::from_((pixel_x, pixel_y), self.w);
            tracing::trace!(name="lov-path-build-bresenham3d-pixel", pixel=?pixel, xy=?xy);

            // Do not compute opacity for last tile, modifier will do it
            if ignore_end_opacity && xy == last_xy {
                tracing::trace!(name="lov-path-build-bresenham3d-ignore-end-opacity", xy=?xy, last_xy=?last_xy);
                break;
            }

            if xy != tile.0 {
                tile.0 = xy;

                if ignored < ignore {
                    tracing::trace!(name="lov-path-build-bresenham3d-ignore", pixel=?pixel, xy=?xy);
                    ignored += 1;
                    continue;
                }

                let mut new_opacity = opacity.0;
                for obj in (self.at)(xy, pixel.z) {
                    new_opacity += obj.opacity.0;
                    tracing::trace!(name="lov-path-build-bresenham3d-obj", pixel=?pixel, xy=?xy, obj=?obj, new_opacity=new_opacity);
                }

                if new_opacity != opacity.0 {
                    let section = Section {
                        start: last,
                        stop: pixel,
                        opacity,
                    };
                    tracing::trace!(name="lov-path-build-bresenham3d-section", pixel=?pixel, xy=?xy, section=?section);
                    sections.push(section);

                    last = pixel;
                    opacity.0 = new_opacity.min(1.0);
                    if opacity.0 >= 1.0 {
                        tracing::trace!(name="lov-path-build-bresenham3d-break", pixel=?pixel, xy=?xy, opacity=?opacity);
                        break;
                    }
                }
            }
        }

        if start != end {
            self.modify(start, end, modifier, &mut opacity);

            let section = Section {
                start: last,
                stop: end,
                opacity,
            };
            tracing::trace!(name="lov-path-build-bresenham3d-end-section", section=?section);
            sections.push(section);
        }

        Path { sections }
    }

    /// Apply some bonus according to "end" tiles
    fn modify(
        &self,
        start: WorldVec3,
        end: WorldVec3,
        modifier: Modifier,
        opacity: &mut CumulatedOpacity,
    ) {
        let end_tile = TileXy::from_([end.x, end.y], self.w).0;
        let radius = self.w.tiles_radius_opacity_picking() as i64;

        let mut best_crawling_bonus: f32 = 0.0;
        let mut best_hiding_bonus: f32 = 0.0;
        let mut best_opacity: f32 = 0.0;

        // Search for best bonuses in "end" area
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                let nx = end_tile.0 as i64 + dx;
                let ny = end_tile.1 as i64 + dy;
                if nx < 0 || ny < 0 {
                    continue;
                }
                let neighbor = TileXy(Xy(nx as u64, ny as u64)).clamped(self.w).0;
                for step in (self.at)(neighbor, end.z) {
                    best_crawling_bonus = best_crawling_bonus.max(step.crawling_opacity_bonus);
                    best_hiding_bonus = best_hiding_bonus.max(step.hiding_opacity_bonus);
                    best_opacity = best_opacity.max(step.opacity.0);
                }
            }
        }

        let (bonus, apply_best_opacity) = match modifier {
            Modifier::Hiding => (best_hiding_bonus, true),
            Modifier::Crawling => (best_crawling_bonus, true),
            Modifier::None(best_opacity) => (0.0, best_opacity),
        };

        if bonus > 0.0 {
            let distance = start.distance(end) / self.w.geo_pixels_per_meters();
            // Bonus is not applicable when watcher is too close
            if distance >= self.w.hide_opacity_bonus_ends().0 {
                // Bonus is applied + bonus per meters
                let bonus = bonus + distance * bonus * self.w.hide_opacity_bonus_meter_factor();
                let best_opacity = apply_best_opacity.then_some(best_opacity).unwrap_or(0.0);
                opacity.0 = (opacity.0 + bonus + best_opacity).min(1.0);
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Path {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub start: WorldVec3,
    pub stop: WorldVec3,
    pub opacity: CumulatedOpacity,
}

impl From<(WorldVec3, WorldVec3, CumulatedOpacity)> for Section {
    fn from(value: (WorldVec3, WorldVec3, CumulatedOpacity)) -> Self {
        Section {
            start: value.0,
            stop: value.1,
            opacity: value.2,
        }
    }
}

#[derive(Debug)]
pub struct Step {
    /// Used to compute new opacity
    pub opacity: Opacity,
    /// Tile nature's crawling opacity bonus
    pub crawling_opacity_bonus: f32,
    /// Tile nature's hiding opacity bonus
    pub hiding_opacity_bonus: f32,
}

impl Step {
    pub fn from_nature(w: &WorldConfig, nature: &Nature, relative_z: f32) -> Self {
        let opacity = nature.opacity(w, relative_z);
        Self {
            opacity,
            crawling_opacity_bonus: nature.crawling_opacity_bonus,
            hiding_opacity_bonus: nature.hiding_opacity_bonus,
        }
    }
}

impl Step {
    pub fn from_smoke(opacity: Percent) -> Self {
        Self {
            opacity: Opacity(opacity.ratio()),
            crawling_opacity_bonus: 0.0,
            hiding_opacity_bonus: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Modifier {
    None(bool), // bool = apply "best opacity" or not
    Hiding,
    Crawling,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_short_path() {
        // Given
        let w = WorldConfig::new(3, 1).with_geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [14., 0., 0.];
        let at = |_, _| {
            vec![Step {
                opacity: Opacity(0.1),
                // solid: false,
                crawling_opacity_bonus: 0.0,
                hiding_opacity_bonus: 0.0,
            }]
        };

        // When
        let path =
            PathBuilder::new(&w, at).build(start.into(), end.into(), 0, Modifier::None(false));

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.].into(),
                        stop: [5., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.),
                    },
                    Section {
                        start: [5., 0., 0.].into(),
                        stop: [10., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.1),
                    },
                    Section {
                        start: [10., 0., 0.].into(),
                        stop: [14., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.2),
                    }
                ]
            }
        )
    }

    #[test]
    fn test_path_with_space() {
        // Given
        let w = WorldConfig::new(6, 1).with_geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [29., 0., 0.];
        let at = |xy, _| match xy {
            Xy(1, 0) | Xy(4, 0) => vec![Step {
                opacity: Opacity(0.1),
                // solid: false,
                crawling_opacity_bonus: 0.0,
                hiding_opacity_bonus: 0.0,
            }],
            _ => vec![Step {
                opacity: Opacity(0.0),
                // solid: false,
                crawling_opacity_bonus: 0.0,
                hiding_opacity_bonus: 0.0,
            }],
        };

        // When
        let path =
            PathBuilder::new(&w, at).build(start.into(), end.into(), 0, Modifier::None(false));

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.].into(),
                        stop: [5., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.),
                    },
                    Section {
                        start: [5., 0., 0.].into(),
                        stop: [20., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.1),
                    },
                    Section {
                        start: [20., 0., 0.].into(),
                        stop: [29., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.2),
                    },
                ]
            }
        )
    }

    #[test]
    fn test_opaque_path() {
        // Given
        let w = WorldConfig::new(3, 1).with_geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [14., 0., 0.];
        let at = |_, _| {
            vec![Step {
                opacity: Opacity(0.6),
                // solid: false,
                crawling_opacity_bonus: 0.0,
                hiding_opacity_bonus: 0.0,
            }]
        };

        // When
        let path =
            PathBuilder::new(&w, at).build(start.into(), end.into(), 0, Modifier::None(false));

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![
                    Section {
                        start: [0., 0., 0.].into(),
                        stop: [5., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.),
                    },
                    Section {
                        start: [5., 0., 0.].into(),
                        stop: [10., 0., 0.].into(),
                        opacity: CumulatedOpacity(0.6),
                    },
                    Section {
                        start: [10., 0., 0.].into(),
                        stop: [14., 0., 0.].into(),
                        opacity: CumulatedOpacity(1.0),
                    }
                ]
            }
        )
    }

    #[test]
    fn test_opaque_path_but_ignore() {
        // Given
        let w = WorldConfig::new(3, 1).with_geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [14., 0., 0.];
        let at = |_, _| {
            vec![Step {
                opacity: Opacity(0.6),
                // solid: false,
                crawling_opacity_bonus: 0.0,
                hiding_opacity_bonus: 0.0,
            }]
        };

        // When
        let path =
            PathBuilder::new(&w, at).build(start.into(), end.into(), 999, Modifier::None(false));

        // Then
        assert_eq!(
            path,
            Path {
                sections: vec![Section {
                    start: WorldVec3 {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0
                    },
                    stop: WorldVec3 {
                        x: 14.0,
                        y: 0.0,
                        z: 0.0
                    },
                    opacity: CumulatedOpacity(0.0),
                }]
            }
        )
    }

    #[test]
    fn test_hiding_bonus_applied_beyond_ends_distance() {
        // Given
        let w = WorldConfig::new(25, 1).with_geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [100., 0., 0.]; // 20 meters away (5 pixels per meter)
        let at = |_, _| {
            vec![Step {
                opacity: Opacity(0.0),
                crawling_opacity_bonus: 0.0,
                hiding_opacity_bonus: 0.2,
            }]
        };

        // When
        let path = PathBuilder::new(&w, at).build(start.into(), end.into(), 0, Modifier::Hiding);

        // Then
        // 0.2 + (20 * 0.2 * 0.005) = 0.22
        assert_eq!(
            path,
            Path {
                sections: vec![Section {
                    start: [0., 0., 0.].into(),
                    stop: [100., 0., 0.].into(),
                    opacity: CumulatedOpacity(0.22),
                }]
            }
        )
    }

    #[test]
    fn test_hiding_bonus_not_applied_below_ends_distance() {
        // Given
        let w = WorldConfig::new(15, 1).with_geo_pixels_per_tile(5);
        let start = [0., 0., 0.];
        let end = [40., 0., 0.]; // 8 meters away, below default hide_opacity_bonus_ends (10 meters)
        let at = |_, _| {
            vec![Step {
                opacity: Opacity(0.0),
                crawling_opacity_bonus: 0.0,
                hiding_opacity_bonus: 0.2,
            }]
        };

        // When (target is hiding)
        let path = PathBuilder::new(&w, at).build(start.into(), end.into(), 0, Modifier::Hiding);

        // Then
        // Too close, no bonus applied
        assert_eq!(
            path,
            Path {
                sections: vec![Section {
                    start: [0., 0., 0.].into(),
                    stop: [40., 0., 0.].into(),
                    opacity: CumulatedOpacity(0.0),
                }]
            }
        )
    }

    #[test]
    fn test_crawling_bonus_picked_from_neighbor_tile_within_radius() {
        // Given
        let w = WorldConfig::new(30, 1)
            .with_geo_pixels_per_tile(5)
            .with_tiles_radius_opacity_picking(1);
        let start = [0., 0., 0.];
        let end = [100., 0., 0.]; // ends on tile Xy(20, 0), 20 meters away
        let at = |xy: Xy, _| match xy {
            Xy(21, 0) => vec![Step {
                opacity: Opacity(0.0),
                crawling_opacity_bonus: 0.3,
                hiding_opacity_bonus: 0.0,
            }],
            _ => vec![Step {
                opacity: Opacity(0.0),
                crawling_opacity_bonus: 0.0,
                hiding_opacity_bonus: 0.0,
            }],
        };

        // When
        let path = PathBuilder::new(&w, at).build(start.into(), end.into(), 0, Modifier::Crawling);

        // Then
        // 0.3 + (20 * 0.3 * 0.005) = 0.33
        assert_eq!(
            path,
            Path {
                sections: vec![Section {
                    start: [0., 0., 0.].into(),
                    stop: [100., 0., 0.].into(),
                    opacity: CumulatedOpacity(0.33),
                }]
            }
        )
    }
}
