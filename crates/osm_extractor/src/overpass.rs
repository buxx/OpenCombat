//! Fetch OSM data through the Overpass API and convert it into [`Feature`]s.

use std::{collections::BTreeMap, thread, time::Duration};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::{
    feature::{DataType, Feature, Geometry, Point, Ring},
    geo::BBox,
};

pub const DEFAULT_OVERPASS_URL: &str = "https://overpass-api.de/api/interpreter";
pub const USER_AGENT: &str = concat!("OpenCombat-osm_extractor/", env!("CARGO_PKG_VERSION"));

const MAX_ATTEMPTS: u32 = 4;

pub struct Overpass {
    client: reqwest::blocking::Client,
    url: String,
}

impl Overpass {
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(300))
            .build()?;
        Ok(Self {
            client,
            url: url.into(),
        })
    }

    /// Fetch features of given (vector) type in the bbox.
    pub fn fetch(&self, type_: DataType, bbox: &BBox) -> Result<Vec<Feature>> {
        let query = query(type_, bbox)?;
        tracing::debug!("Overpass query: {query}");
        let response = self.post(&query)?;
        let response: Response =
            serde_json::from_str(&response).context("parse Overpass response")?;
        if let Some(remark) = &response.remark {
            tracing::warn!("Overpass remark: {remark}");
        }
        Ok(response
            .elements
            .into_iter()
            .filter_map(|element| to_feature(type_, element))
            .collect())
    }

    fn post(&self, query: &str) -> Result<String> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            let result = self
                .client
                .post(&self.url)
                .form(&[("data", query)])
                .send()
                .and_then(|r| r.error_for_status())
                .and_then(|r| r.text());
            match result {
                Ok(text) => return Ok(text),
                Err(error) if attempt < MAX_ATTEMPTS && is_retryable(&error) => {
                    let wait = Duration::from_secs(5 * 2u64.pow(attempt));
                    tracing::warn!("Overpass request failed ({error}), retry in {wait:?}");
                    thread::sleep(wait);
                }
                Err(error) => return Err(error).context("Overpass request"),
            }
        }
    }
}

fn is_retryable(error: &reqwest::Error) -> bool {
    error.is_timeout()
        || error.is_connect()
        || error
            .status()
            .is_some_and(|s| s.as_u16() == 429 || s.is_server_error())
}

/// Build the Overpass QL query for a data type.
pub fn query(type_: DataType, bbox: &BBox) -> Result<String> {
    let filters: &[&str] = match type_ {
        DataType::Road => &[r#"way["highway"]"#],
        DataType::Building => &[r#"way["building"]"#, r#"relation["building"]"#],
        DataType::Water => &[
            r#"way["natural"="water"]"#,
            r#"relation["natural"="water"]"#,
            r#"way["waterway"]"#,
            r#"relation["waterway"="riverbank"]"#,
            r#"way["landuse"~"^(reservoir|basin)$"]"#,
            r#"relation["landuse"~"^(reservoir|basin)$"]"#,
        ],
        DataType::Forest => &[
            r#"way["landuse"="forest"]"#,
            r#"relation["landuse"="forest"]"#,
            r#"way["natural"="wood"]"#,
            r#"relation["natural"="wood"]"#,
        ],
        DataType::Height => bail!("height is not an OSM data type"),
    };
    let bbox = bbox.overpass();
    let body: String = filters.iter().map(|f| format!("{f}({bbox});")).collect();
    Ok(format!("[out:json][timeout:240];({body});out geom;"))
}

#[derive(Debug, Deserialize)]
struct Response {
    #[serde(default)]
    elements: Vec<Element>,
    #[serde(default)]
    remark: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Element {
    Node {},
    Way {
        id: u64,
        #[serde(default)]
        tags: BTreeMap<String, String>,
        #[serde(default)]
        geometry: Vec<Option<Coord>>,
    },
    Relation {
        id: u64,
        #[serde(default)]
        tags: BTreeMap<String, String>,
        #[serde(default)]
        members: Vec<Member>,
    },
}

#[derive(Debug, Deserialize)]
struct Member {
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    role: String,
    #[serde(default)]
    geometry: Vec<Option<Coord>>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct Coord {
    lat: f64,
    lon: f64,
}

fn points(geometry: &[Option<Coord>]) -> Vec<Point> {
    geometry.iter().flatten().map(|c| [c.lon, c.lat]).collect()
}

fn is_closed(points: &[Point]) -> bool {
    points.len() >= 4 && points.first() == points.last()
}

/// Whether a way of this type must be considered as an area (polygon) rather than a line.
fn way_is_area(type_: DataType, tags: &BTreeMap<String, String>, closed: bool) -> bool {
    if !closed {
        return false;
    }
    let tag = |k: &str| tags.get(k).map(String::as_str);
    match type_ {
        DataType::Road => tag("area") == Some("yes"),
        DataType::Water => {
            tag("natural") == Some("water")
                || tag("waterway") == Some("riverbank")
                || matches!(tag("landuse"), Some("reservoir" | "basin"))
                || tag("area") == Some("yes")
        }
        DataType::Building | DataType::Forest => true,
        DataType::Height => false,
    }
}

fn to_feature(type_: DataType, element: Element) -> Option<Feature> {
    match element {
        Element::Node {} => None,
        Element::Way { id, tags, geometry } => {
            if type_ == DataType::Road
                && matches!(
                    tags.get("highway").map(String::as_str),
                    Some("proposed" | "construction" | "platform" | "abandoned" | "razed")
                )
            {
                return None;
            }
            let points = points(&geometry);
            if points.len() < 2 {
                return None;
            }
            let geometry = if way_is_area(type_, &tags, is_closed(&points)) {
                Geometry::Polygon(vec![points])
            } else if matches!(type_, DataType::Building | DataType::Forest) {
                // Unclosed area: broken data, ignore
                return None;
            } else {
                Geometry::LineString(points)
            };
            Some(Feature::new(format!("way/{id}"), tags, geometry))
        }
        Element::Relation { id, tags, members } => {
            if tags.get("type").map(String::as_str) != Some("multipolygon") {
                return None;
            }
            let ways = |role: &str| -> Vec<Vec<Point>> {
                members
                    .iter()
                    .filter(|m| m.type_ == "way" && m.role == role)
                    .map(|m| points(&m.geometry))
                    .filter(|p| p.len() >= 2)
                    .collect()
            };
            let mut outers = ways("outer");
            // Untagged role members are considered as outer
            outers.extend(ways(""));
            let outers = stitch_rings(outers);
            let inners = stitch_rings(ways("inner"));
            let polygons = assemble_multipolygon(outers, inners);
            if polygons.is_empty() {
                return None;
            }
            Some(Feature::new(
                format!("relation/{id}"),
                tags,
                Geometry::MultiPolygon(polygons),
            ))
        }
    }
}

/// Join way segments sharing end points into closed rings.
pub fn stitch_rings(mut segments: Vec<Vec<Point>>) -> Vec<Ring> {
    let mut rings = vec![];
    while let Some(mut ring) = segments.pop() {
        while !is_closed(&ring) {
            let end = *ring.last().expect("ring has points");
            let found = segments
                .iter()
                .position(|s| s.first() == Some(&end) || s.last() == Some(&end));
            let Some(index) = found else {
                break;
            };
            let mut next = segments.swap_remove(index);
            if next.first() != Some(&end) {
                next.reverse();
            }
            ring.extend(next.into_iter().skip(1));
        }
        if !is_closed(&ring) {
            // Incomplete ring (ex. relation partially outside of downloaded data): close it anyway
            let first = ring[0];
            ring.push(first);
        }
        if ring.len() >= 4 {
            rings.push(ring);
        }
    }
    rings
}

/// Attach each inner ring to the outer ring containing it.
pub fn assemble_multipolygon(outers: Vec<Ring>, inners: Vec<Ring>) -> Vec<Vec<Ring>> {
    let mut polygons: Vec<Vec<Ring>> = outers.into_iter().map(|o| vec![o]).collect();
    for inner in inners {
        let probe = inner[0];
        if let Some(polygon) = polygons.iter_mut().find(|p| point_in_ring(probe, &p[0])) {
            polygon.push(inner);
        }
    }
    polygons
}

fn point_in_ring(p: Point, ring: &[Point]) -> bool {
    let mut inside = false;
    let mut j = ring.len() - 1;
    for i in 0..ring.len() {
        let (a, b) = (ring[i], ring[j]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stitch_two_halves() {
        let a = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]];
        let b = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]; // reversed direction
        let rings = stitch_rings(vec![a, b]);
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 5);
        assert!(is_closed(&rings[0]));
    }

    #[test]
    fn inner_goes_to_containing_outer() {
        let outer1 = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
            [0.0, 0.0],
        ];
        let outer2 = vec![
            [20.0, 0.0],
            [30.0, 0.0],
            [30.0, 10.0],
            [20.0, 10.0],
            [20.0, 0.0],
        ];
        let inner = vec![[22.0, 2.0], [24.0, 2.0], [24.0, 4.0], [22.0, 2.0]];
        let polygons = assemble_multipolygon(vec![outer1, outer2], vec![inner]);
        assert_eq!(polygons[0].len(), 1);
        assert_eq!(polygons[1].len(), 2);
    }

    #[test]
    fn parse_elements() {
        let json = r#"{"elements":[
            {"type":"way","id":1,"tags":{"building":"yes"},"geometry":[{"lat":0,"lon":0},{"lat":0,"lon":1},{"lat":1,"lon":1},{"lat":0,"lon":0}]},
            {"type":"way","id":2,"tags":{"waterway":"stream"},"geometry":[{"lat":0,"lon":0},{"lat":0,"lon":1}]},
            {"type":"node","id":3,"lat":0,"lon":0}
        ]}"#;
        let response: Response = serde_json::from_str(json).unwrap();
        let features: Vec<_> = response
            .elements
            .into_iter()
            .filter_map(|e| to_feature(DataType::Water, e))
            .collect();
        assert_eq!(features.len(), 2);
        assert!(matches!(features[1].geometry, Geometry::LineString(_)));
    }
}
