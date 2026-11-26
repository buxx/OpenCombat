use std::{f64::consts::PI, fmt, str::FromStr};

use serde::{Deserialize, Serialize};

pub const EARTH_RADIUS_M: f64 = 6_378_137.0;

/// A GPS coordinate (WGS84).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

impl FromStr for LatLon {
    type Err = String;

    /// Parse `"lat,lon"` (ex: `"49.3410,-0.8790"`).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (lat, lon) = s
            .split_once(',')
            .ok_or_else(|| format!("expected 'lat,lon', got '{s}'"))?;
        let lat: f64 = lat
            .trim()
            .parse()
            .map_err(|e| format!("invalid latitude '{lat}': {e}"))?;
        let lon: f64 = lon
            .trim()
            .parse()
            .map_err(|e| format!("invalid longitude '{lon}': {e}"))?;
        if !(-85.0..=85.0).contains(&lat) {
            return Err(format!("latitude {lat} out of range [-85, 85]"));
        }
        if !(-180.0..=180.0).contains(&lon) {
            return Err(format!("longitude {lon} out of range [-180, 180]"));
        }
        Ok(Self { lat, lon })
    }
}

/// The four corners of an area, as given by the user.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corners {
    pub top_left: LatLon,
    pub top_right: LatLon,
    pub bottom_left: LatLon,
    pub bottom_right: LatLon,
}

impl Corners {
    /// Check corners are consistent: top corners north of bottom ones, left corners west of right ones.
    pub fn validate(&self) -> Result<(), String> {
        let checks = [
            (
                self.top_left.lat > self.bottom_left.lat,
                "top left must be north of bottom left",
            ),
            (
                self.top_right.lat > self.bottom_right.lat,
                "top right must be north of bottom right",
            ),
            (
                self.top_left.lon < self.top_right.lon,
                "top left must be west of top right",
            ),
            (
                self.bottom_left.lon < self.bottom_right.lon,
                "bottom left must be west of bottom right",
            ),
        ];
        match checks.iter().find(|(ok, _)| !ok) {
            Some((_, message)) => Err(message.to_string()),
            None => Ok(()),
        }
    }

    fn all(&self) -> [LatLon; 4] {
        [
            self.top_left,
            self.top_right,
            self.bottom_left,
            self.bottom_right,
        ]
    }

    /// Smallest north-aligned bbox containing the four corners.
    pub fn bbox(&self) -> BBox {
        let all = self.all();
        BBox {
            south: all.iter().map(|p| p.lat).fold(f64::INFINITY, f64::min),
            west: all.iter().map(|p| p.lon).fold(f64::INFINITY, f64::min),
            north: all.iter().map(|p| p.lat).fold(f64::NEG_INFINITY, f64::max),
            east: all.iter().map(|p| p.lon).fold(f64::NEG_INFINITY, f64::max),
        }
    }

    /// Max distance, in meters, between a given corner and the matching corner of [`Self::bbox`].
    /// Zero when the corners form a north-aligned rectangle.
    pub fn max_deviation_m(&self) -> f64 {
        let bbox = self.bbox();
        let lon_scale = EARTH_RADIUS_M * ((bbox.south + bbox.north) / 2.0).to_radians().cos();
        let distance = |p: LatLon, lat: f64, lon: f64| {
            let dy = (p.lat - lat).to_radians() * EARTH_RADIUS_M;
            let dx = (p.lon - lon).to_radians() * lon_scale;
            dx.hypot(dy)
        };
        [
            distance(self.top_left, bbox.north, bbox.west),
            distance(self.top_right, bbox.north, bbox.east),
            distance(self.bottom_left, bbox.south, bbox.west),
            distance(self.bottom_right, bbox.south, bbox.east),
        ]
        .into_iter()
        .fold(0.0, f64::max)
    }
}

/// Geographic bounding box (WGS84 degrees).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

impl BBox {
    /// Overpass bbox filter: `(south,west,north,east)`.
    pub fn overpass(&self) -> String {
        format!("{},{},{},{}", self.south, self.west, self.north, self.east)
    }

    /// Approximate ground size in meters (width, height), at the bbox center latitude.
    pub fn size_meters(&self) -> (f64, f64) {
        let scale = EARTH_RADIUS_M * ((self.south + self.north) / 2.0).to_radians().cos();
        let width = (self.east - self.west).to_radians() * scale;
        let height = (merc_y(self.north) - merc_y(self.south)) * scale;
        (width, height)
    }

    pub fn is_empty(&self) -> bool {
        self.north <= self.south || self.east <= self.west
    }
}

impl fmt::Display for BBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "S{:.6} W{:.6} N{:.6} E{:.6}",
            self.south, self.west, self.north, self.east
        )
    }
}

/// Web Mercator Y (radians-like, unscaled) for a latitude in degrees.
pub fn merc_y(lat: f64) -> f64 {
    let lat = lat.to_radians();
    (PI / 4.0 + lat / 2.0).tan().ln()
}

/// Fractional slippy-map tile coordinates for a point at given zoom.
pub fn tile_xy(lat: f64, lon: f64, zoom: u8) -> (f64, f64) {
    let n = f64::from(1u32 << zoom);
    let x = (lon + 180.0) / 360.0 * n;
    let y = (1.0 - merc_y(lat) / PI) / 2.0 * n;
    (x, y)
}

/// Linear projection of a bbox onto an image of `width` x `height` pixels (Web Mercator).
#[derive(Debug, Clone, Copy)]
pub struct Projector {
    bbox: BBox,
    merc_north: f64,
    merc_south: f64,
    pub width: u32,
    pub height: u32,
}

impl Projector {
    pub fn new(bbox: BBox, width: u32, height: u32) -> Self {
        Self {
            bbox,
            merc_north: merc_y(bbox.north),
            merc_south: merc_y(bbox.south),
            width,
            height,
        }
    }

    /// Project `[lon, lat]` to pixel `(x, y)`.
    pub fn project(&self, p: [f64; 2]) -> (f32, f32) {
        let [lon, lat] = p;
        let x = (lon - self.bbox.west) / (self.bbox.east - self.bbox.west) * f64::from(self.width);
        let y = (self.merc_north - merc_y(lat)) / (self.merc_north - self.merc_south)
            * f64::from(self.height);
        (x as f32, y as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_latlon() {
        let p: LatLon = "49.34,-0.88".parse().unwrap();
        assert_eq!(
            p,
            LatLon {
                lat: 49.34,
                lon: -0.88
            }
        );
        assert!("49.34".parse::<LatLon>().is_err());
        assert!("95,0".parse::<LatLon>().is_err());
    }

    fn ll(lat: f64, lon: f64) -> LatLon {
        LatLon { lat, lon }
    }

    #[test]
    fn aligned_corners() {
        let corners = Corners {
            top_left: ll(2.0, 1.0),
            top_right: ll(2.0, 3.0),
            bottom_left: ll(1.0, 1.0),
            bottom_right: ll(1.0, 3.0),
        };
        assert!(corners.validate().is_ok());
        assert_eq!(
            corners.bbox(),
            BBox {
                south: 1.0,
                west: 1.0,
                north: 2.0,
                east: 3.0
            }
        );
        assert!(corners.max_deviation_m() < 1e-6);
    }

    #[test]
    fn rotated_corners_use_envelope() {
        let corners = Corners {
            top_left: ll(2.0, 1.0),
            top_right: ll(2.1, 3.0),
            bottom_left: ll(0.9, 1.0),
            bottom_right: ll(1.0, 3.0),
        };
        assert!(corners.validate().is_ok());
        assert_eq!(
            corners.bbox(),
            BBox {
                south: 0.9,
                west: 1.0,
                north: 2.1,
                east: 3.0
            }
        );
        assert!(corners.max_deviation_m() > 10_000.0);
    }

    #[test]
    fn inverted_corners_rejected() {
        let corners = Corners {
            top_left: ll(1.0, 1.0),
            top_right: ll(1.0, 3.0),
            bottom_left: ll(2.0, 1.0),
            bottom_right: ll(2.0, 3.0),
        };
        assert!(corners.validate().is_err());
    }

    #[test]
    fn projector_corners() {
        let bbox = BBox {
            south: 49.0,
            west: -1.0,
            north: 49.01,
            east: -0.99,
        };
        let proj = Projector::new(bbox, 100, 200);
        let (x, y) = proj.project([-1.0, 49.01]);
        assert!(x.abs() < 1e-3 && y.abs() < 1e-3);
        let (x, y) = proj.project([-0.99, 49.0]);
        assert!((x - 100.0).abs() < 1e-3 && (y - 200.0).abs() < 1e-3);
    }
}
