//! Extracted data model, stored on disk as GeoJSON (coordinates are `[lon, lat]`).

use std::{collections::BTreeMap, fmt, path::Path, str::FromStr};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Kind of data which can be extracted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DataType {
    Road,
    Building,
    Water,
    Forest,
    Height,
}

impl DataType {
    pub const ALL: [DataType; 5] = [
        DataType::Road,
        DataType::Building,
        DataType::Water,
        DataType::Forest,
        DataType::Height,
    ];

    /// Vector (OSM) layers, in drawing order (bottom to top).
    pub const LAYERS: [DataType; 4] = [
        DataType::Forest,
        DataType::Water,
        DataType::Building,
        DataType::Road,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            DataType::Road => "road",
            DataType::Building => "building",
            DataType::Water => "water",
            DataType::Forest => "forest",
            DataType::Height => "height",
        }
    }

    /// GeoJSON file name for vector layers.
    pub fn geojson_file(&self) -> String {
        format!("{}.geojson", self.name())
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for DataType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "road" | "roads" => Ok(DataType::Road),
            "building" | "buildings" => Ok(DataType::Building),
            "water" => Ok(DataType::Water),
            "forest" | "forests" | "wood" => Ok(DataType::Forest),
            "height" | "elevation" | "dem" => Ok(DataType::Height),
            other => Err(format!(
                "unknown type '{other}' (expected road, building, water, forest or height)"
            )),
        }
    }
}

pub type Point = [f64; 2];
pub type Ring = Vec<Point>;

/// GeoJSON geometry. Polygons are lists of rings, the first one being the outer ring.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "coordinates")]
pub enum Geometry {
    LineString(Vec<Point>),
    Polygon(Vec<Ring>),
    MultiPolygon(Vec<Vec<Ring>>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    #[serde(rename = "type")]
    pub type_: FeatureTag,
    /// `way/123` or `relation/456`.
    pub id: String,
    /// OSM tags.
    pub properties: BTreeMap<String, String>,
    pub geometry: Geometry,
}

impl Feature {
    pub fn new(id: String, properties: BTreeMap<String, String>, geometry: Geometry) -> Self {
        Self {
            type_: FeatureTag::Feature,
            id,
            properties,
            geometry,
        }
    }

    pub fn tag(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(String::as_str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FeatureTag {
    Feature,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FeatureCollectionTag {
    FeatureCollection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub type_: FeatureCollectionTag,
    pub features: Vec<Feature>,
}

impl FeatureCollection {
    pub fn new(features: Vec<Feature>) -> Self {
        Self {
            type_: FeatureCollectionTag::FeatureCollection,
            features,
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
        serde_json::from_reader(std::io::BufReader::new(file))
            .with_context(|| format!("parse {}", path.display()))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file =
            std::fs::File::create(path).with_context(|| format!("create {}", path.display()))?;
        serde_json::to_writer(std::io::BufWriter::new(file), self)
            .with_context(|| format!("write {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geojson_roundtrip() {
        let collection = FeatureCollection::new(vec![Feature::new(
            "way/1".into(),
            BTreeMap::from([("highway".into(), "primary".into())]),
            Geometry::LineString(vec![[1.0, 2.0], [3.0, 4.0]]),
        )]);
        let json = serde_json::to_value(&collection).unwrap();
        assert_eq!(json["type"], "FeatureCollection");
        assert_eq!(json["features"][0]["type"], "Feature");
        assert_eq!(json["features"][0]["geometry"]["type"], "LineString");
        assert_eq!(json["features"][0]["geometry"]["coordinates"][1][0], 3.0);
        let back: FeatureCollection = serde_json::from_value(json).unwrap();
        assert_eq!(back, collection);
    }
}
