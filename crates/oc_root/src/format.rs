//! Read and write values from/to files in the formats used by oc (RON, YAML, TOML, rkyv), with a
//! common error type carrying the file path.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use rkyv::{
    Archive, Deserialize, Serialize,
    api::high::{HighDeserializer, HighSerializer, HighValidator},
    bytecheck::CheckBytes,
    rancor,
    ser::allocator::ArenaHandle,
    util::AlignedVec,
};
use serde::{Serialize as SerdeSerialize, de::DeserializeOwned};

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("Read/write '{path}': {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("Format of '{path}': {message}")]
    Format { path: PathBuf, message: String },
}

impl FileError {
    pub fn io(path: &Path, source: io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }

    pub fn format(path: &Path, message: impl ToString) -> Self {
        Self::Format {
            path: path.to_path_buf(),
            message: message.to_string(),
        }
    }
}

/// A text serialization format
pub trait TextFormat {
    fn parse<T: DeserializeOwned>(content: &str) -> Result<T, String>;
    fn render<T: SerdeSerialize>(value: &T) -> Result<String, String>;
}

pub struct Ron;
pub struct Yaml;
pub struct Toml;

impl TextFormat for Ron {
    fn parse<T: DeserializeOwned>(content: &str) -> Result<T, String> {
        ron::from_str(content).map_err(|e| e.to_string())
    }

    fn render<T: SerdeSerialize>(value: &T) -> Result<String, String> {
        ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())
    }
}

impl TextFormat for Yaml {
    fn parse<T: DeserializeOwned>(content: &str) -> Result<T, String> {
        serde_yaml::from_str(content).map_err(|e| e.to_string())
    }

    fn render<T: SerdeSerialize>(value: &T) -> Result<String, String> {
        serde_yaml::to_string(value).map_err(|e| e.to_string())
    }
}

impl TextFormat for Toml {
    fn parse<T: DeserializeOwned>(content: &str) -> Result<T, String> {
        toml::from_str(content).map_err(|e| e.to_string())
    }

    fn render<T: SerdeSerialize>(value: &T) -> Result<String, String> {
        toml::to_string_pretty(value).map_err(|e| e.to_string())
    }
}

/// Read and parse file at `path` with format `F`
pub fn read<F: TextFormat, T: DeserializeOwned>(path: &Path) -> Result<T, FileError> {
    let content = fs::read_to_string(path).map_err(|e| FileError::io(path, e))?;
    F::parse(&content).map_err(|e| FileError::format(path, e))
}

/// Serialize `value` with format `F` and write it to file at `path` (creating parent directories)
pub fn write<F: TextFormat, T: SerdeSerialize>(path: &Path, value: &T) -> Result<(), FileError> {
    let content = F::render(value).map_err(|e| FileError::format(path, e))?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| FileError::io(parent, e))?;
    }
    fs::write(path, content).map_err(|e| FileError::io(path, e))
}

pub fn read_ron<T: DeserializeOwned>(path: &Path) -> Result<T, FileError> {
    read::<Ron, T>(path)
}

pub fn write_ron<T: SerdeSerialize>(path: &Path, value: &T) -> Result<(), FileError> {
    write::<Ron, T>(path, value)
}

pub fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T, FileError> {
    read::<Yaml, T>(path)
}

pub fn write_yaml<T: SerdeSerialize>(path: &Path, value: &T) -> Result<(), FileError> {
    write::<Yaml, T>(path, value)
}

pub fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T, FileError> {
    read::<Toml, T>(path)
}

/// Value which can be encoded with rkyv
pub trait Encode:
    for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rancor::Error>>
{
}
impl<T> Encode for T where
    T: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, rancor::Error>>
{
}

/// Value which can be decoded (validated) with rkyv
pub trait Decode: Archive<Archived = Self::Arch> + Sized {
    type Arch: for<'a> CheckBytes<HighValidator<'a, rancor::Error>>
        + Deserialize<Self, HighDeserializer<rancor::Error>>;
}
impl<T> Decode for T
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, rancor::Error>>
        + Deserialize<T, HighDeserializer<rancor::Error>>,
{
    type Arch = T::Archived;
}

/// Encode `value` into rkyv bytes
pub fn encode<T: Encode>(value: &T) -> Result<AlignedVec, rancor::Error> {
    rkyv::to_bytes::<rancor::Error>(value)
}

/// Decode rkyv `bytes` (which don't need to be aligned)
pub fn decode<T: Decode>(bytes: &[u8]) -> Result<T, rancor::Error> {
    let mut aligned: AlignedVec = AlignedVec::with_capacity(bytes.len());
    aligned.extend_from_slice(bytes);
    rkyv::from_bytes::<T, rancor::Error>(&aligned)
}

/// Load from and save to rkyv files. Implemented for every encodable and decodable type.
pub trait RkyvFile: Sized {
    fn load(path: &Path) -> Result<Self, FileError>;
    fn save(&self, path: &Path) -> Result<(), FileError>;
}

impl<T: Encode + Decode> RkyvFile for T {
    fn load(path: &Path) -> Result<Self, FileError> {
        let bytes = fs::read(path).map_err(|e| FileError::io(path, e))?;
        decode(&bytes).map_err(|e| FileError::format(path, e))
    }

    fn save(&self, path: &Path) -> Result<(), FileError> {
        let bytes = encode(self).map_err(|e| FileError::format(path, e))?;
        fs::write(path, &bytes).map_err(|e| FileError::io(path, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(
        Debug,
        PartialEq,
        serde::Serialize,
        serde::Deserialize,
        rkyv::Archive,
        rkyv::Serialize,
        rkyv::Deserialize,
    )]
    struct Value {
        name: String,
        count: u32,
    }

    fn value() -> Value {
        Value {
            name: "a".into(),
            count: 3,
        }
    }

    #[test]
    fn text_formats_round_trip() {
        let dir = std::env::temp_dir().join(format!("oc_root_format_{}", std::process::id()));
        let ron_ = dir.join("value.ron");
        let yaml = dir.join("sub/value.yaml");
        write_ron(&ron_, &value()).unwrap();
        write_yaml(&yaml, &value()).unwrap();
        assert_eq!(read_ron::<Value>(&ron_).unwrap(), value());
        assert_eq!(read_yaml::<Value>(&yaml).unwrap(), value());
        assert!(matches!(
            read_toml::<Value>(&ron_),
            Err(FileError::Format { .. })
        ));
        assert!(matches!(
            read_ron::<Value>(&dir.join("missing.ron")),
            Err(FileError::Io { .. })
        ));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rkyv_round_trip() {
        assert_eq!(
            decode::<Value>(&encode(&value()).unwrap()).unwrap(),
            value()
        );

        let path = std::env::temp_dir().join(format!("oc_root_rkyv_{}", std::process::id()));
        value().save(&path).unwrap();
        assert_eq!(Value::load(&path).unwrap(), value());
        fs::remove_file(path).unwrap();
    }
}
