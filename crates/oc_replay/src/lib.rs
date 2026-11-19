use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    net::SocketAddr,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use oc_geo::region::WorldRegionIndex;
use oc_network::{GameConfig, ToClient, ToServer};
use oc_root::{
    files::Roots,
    format::{decode, encode},
    identity::Identity,
    side::Side,
};

const MAGIC: &[u8; 5] = b"OCREC";
const VERSION: u8 = 2;
const SIDE_A: u8 = 0;
const SIDE_B: u8 = 1;
const SERVER: u8 = 0;
const LOCAL: u8 = 1;

const FLUSH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Io: {0}")]
    Io(#[from] io::Error),
    #[error("Not a record file")]
    NotARecord,
    #[error("Unsupported version {0} (expected {1})")]
    UnsupportedVersion(u8, u8),
    #[error("Invalid header: {0}")]
    Header(String),
    #[error("Encode/decode message: {0}")]
    Format(String),
}

/// Record context, needed by replay
#[derive(Debug, Clone)]
pub struct Header {
    /// Side of the recorder (side specific messages, like supports count, are for this side)
    pub side: Side,
    /// Where replay finds world and mod files
    pub source: Source,
    /// Record start, in seconds since unix epoch
    pub recorded: u64,
    /// Mod during record
    pub mod_: Named,
    /// Map during record
    pub map: Named,
}

/// Name and version of a mod or a map
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    pub name: String,
    pub version: u32,
}

impl std::fmt::Display for Named {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

impl Header {
    /// Header of a record starting now, from game config (first message sent by server)
    pub fn from_config(side: Side, source: Source, config: &GameConfig) -> Self {
        let recorded = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        Self {
            side,
            source,
            recorded,
            mod_: Named {
                name: config.mod_.name().to_string(),
                version: config.mod_.version(),
            },
            map: Named {
                name: config.meta.name.clone(),
                version: config.meta.revision,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Source {
    /// Recorded from a network server: files are the ones downloaded from it
    Server(SocketAddr),
    /// Recorded from an embedded server: files are local
    Local(Roots),
}

#[derive(Debug, Clone)]
pub struct Frame {
    /// Since record start
    pub elapsed: Duration,
    /// The message to record
    pub message: ToClient,
}

pub struct Writer {
    file: BufWriter<File>,
}

impl Writer {
    pub fn create(path: &Path, header: &Header) -> Result<Self, Error> {
        let mut file = BufWriter::new(File::create(path)?);

        file.write_all(MAGIC)?;
        file.write_all(&[VERSION])?;
        file.write_all(&[match header.side {
            Side::A => SIDE_A,
            Side::B => SIDE_B,
        }])?;
        match &header.source {
            Source::Server(address) => {
                file.write_all(&[SERVER])?;
                write_str(&mut file, &address.to_string())?;
            }
            Source::Local(roots) => {
                file.write_all(&[LOCAL])?;
                for path in [&roots.mods, &roots.worlds, &roots.derived, &roots.archives] {
                    write_str(&mut file, &path.to_string_lossy())?;
                }
            }
        }
        file.write_all(&header.recorded.to_le_bytes())?;
        for named in [&header.mod_, &header.map] {
            write_str(&mut file, &named.name)?;
            file.write_all(&named.version.to_le_bytes())?;
        }

        Ok(Self { file })
    }

    pub fn write(&mut self, elapsed: Duration, message: &ToClient) -> Result<(), Error> {
        let bytes = encode(message).map_err(|e| Error::Format(e.to_string()))?;
        let elapsed = (elapsed.as_micros() as u64).to_le_bytes();
        self.file.write_all(&elapsed)?;
        self.file.write_all(&(bytes.len() as u32).to_le_bytes())?;
        self.file.write_all(&bytes)?;

        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        Ok(self.file.flush()?)
    }
}

pub struct Recorder {
    path: PathBuf,
    source: Source,
    writer: Option<Writer>,
    identity: Identity,
    start: Option<Instant>,
    last_flush: Instant,
    count: u64,
    ended: bool,
}

impl Recorder {
    /// Record file at `path` is created at game config reception
    pub fn new(path: &Path, side: Side, source: Source) -> Self {
        let identity = Identity {
            uuid: uuid::Uuid::new_v4(),
            side,
        };
        Self {
            path: path.to_path_buf(),
            source,
            writer: None,
            identity,
            start: None,
            last_flush: Instant::now(),
            count: 0,
            ended: false,
        }
    }

    pub fn record(&mut self, message: &ToClient) -> Result<Vec<ToServer>, Error> {
        if self.ended {
            return Ok(vec![]);
        }

        // Server send game config at connection: record starts here
        if let ToClient::GameConfig(config) = message
            && self.writer.is_none()
        {
            let header = Header::from_config(self.identity.side, self.source.clone(), config);
            self.writer = Some(Writer::create(&self.path, &header)?);
        }

        let Some(writer) = &mut self.writer else {
            tracing::warn!("Record: message received before game config, ignored");
            return Ok(vec![]);
        };

        let start = *self.start.get_or_insert_with(Instant::now);
        writer.write(start.elapsed(), message)?;
        self.count += 1;

        // Only when receive Gameconfig to reply with init and set regions listening
        let mut replies = vec![];

        match message {
            ToClient::GameConfig(config) => {
                let regions = config.w.regions_count();
                tracing::info!("Record: received game config, listen {regions} regions");
                replies.push(ToServer::RequestInit(self.identity.clone()));
                replies.extend((0..regions).map(|r| ToServer::ListenRegion(WorldRegionIndex(r))));
            }
            ToClient::EndResume(_) => {
                tracing::info!("Record: battle ended ({} messages)", self.count);
                self.ended = true;
                self.flush()?;
            }
            _ => {}
        }

        if self.last_flush.elapsed() >= FLUSH_INTERVAL {
            self.flush()?;
        }

        Ok(replies)
    }

    pub fn ended(&self) -> bool {
        self.ended
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.last_flush = Instant::now();
        match &mut self.writer {
            Some(writer) => writer.flush(),
            None => Ok(()),
        }
    }
}

pub struct Reader {
    file: BufReader<File>,
    header: Header,
}

impl Reader {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let mut file = BufReader::new(File::open(path)?);

        let mut magic = [0u8; 6];
        file.read_exact(&mut magic)?;
        if &magic[..5] != MAGIC {
            return Err(Error::NotARecord);
        }
        if magic[5] != VERSION {
            return Err(Error::UnsupportedVersion(magic[5], VERSION));
        }

        let side = match read_array::<1>(&mut file)?[0] {
            SIDE_A => Side::A,
            SIDE_B => Side::B,
            other => return Err(Error::Header(format!("unknown side {other}"))),
        };
        let source = match read_array::<1>(&mut file)?[0] {
            SERVER => Source::Server(
                read_str(&mut file)?
                    .parse()
                    .map_err(|e: std::net::AddrParseError| Error::Header(e.to_string()))?,
            ),
            LOCAL => {
                let mut path = || read_str(&mut file).map(PathBuf::from);
                Source::Local(Roots {
                    mods: path()?,
                    worlds: path()?,
                    derived: path()?,
                    archives: path()?,
                })
            }
            other => return Err(Error::Header(format!("unknown source {other}"))),
        };
        let recorded = u64::from_le_bytes(read_array(&mut file)?);
        let mut named = || -> Result<Named, Error> {
            Ok(Named {
                name: read_str(&mut file)?,
                version: u32::from_le_bytes(read_array(&mut file)?),
            })
        };
        let (mod_, map) = (named()?, named()?);

        Ok(Self {
            file,
            header: Header {
                side,
                source,
                recorded,
                mod_,
                map,
            },
        })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Header of the record at `path` (frames are not read)
    pub fn read_header(path: &Path) -> Result<Header, Error> {
        Ok(Self::open(path)?.header)
    }

    // WARNING: AI generated
    /// Elapsed time of the last frame of the record at `path` (messages are not decoded)
    pub fn duration(path: &Path) -> Result<Duration, Error> {
        let mut reader = Self::open(path)?;
        let mut duration = Duration::ZERO;
        loop {
            let (elapsed, len) = match (
                read_array::<8>(&mut reader.file),
                read_array::<4>(&mut reader.file),
            ) {
                (Ok(elapsed), Ok(len)) => (elapsed, len),
                _ => return Ok(duration),
            };
            let len = u32::from_le_bytes(len) as i64;
            if reader.file.seek_relative(len).is_err() {
                return Ok(duration);
            }
            duration = Duration::from_micros(u64::from_le_bytes(elapsed));
        }
    }

    // WARNING: AI generated
    /// Next frame, or None at end of file (a truncated last frame is considered as end of file)
    pub fn next_frame(&mut self) -> Result<Option<Frame>, Error> {
        let elapsed = match read_array::<8>(&mut self.file) {
            Ok(bytes) => Duration::from_micros(u64::from_le_bytes(bytes)),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let len = match read_array::<4>(&mut self.file) {
            Ok(bytes) => u32::from_le_bytes(bytes) as usize,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let mut bytes = vec![0u8; len];
        match self.file.read_exact(&mut bytes) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        let message = decode::<ToClient>(&bytes).map_err(|e| Error::Format(e.to_string()))?;

        Ok(Some(Frame { elapsed, message }))
    }
}

impl Iterator for Reader {
    type Item = Result<Frame, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_frame().transpose()
    }
}

// WARNING: AI generated
fn write_str(file: &mut impl Write, value: &str) -> io::Result<()> {
    file.write_all(&(value.len() as u32).to_le_bytes())?;
    file.write_all(value.as_bytes())
}

// WARNING: AI generated
fn read_str(file: &mut impl Read) -> Result<String, Error> {
    let len = u32::from_le_bytes(read_array(file)?) as usize;
    let mut bytes = vec![0u8; len];
    file.read_exact(&mut bytes)?;
    String::from_utf8(bytes).map_err(|e| Error::Header(e.to_string()))
}

// WARNING: AI generated
fn read_array<const N: usize>(file: &mut impl Read) -> io::Result<[u8; N]> {
    let mut bytes = [0u8; N];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_then_read() {
        let path = std::env::temp_dir().join(format!("oc_replay_{}.ocrec", std::process::id()));
        let header = Header {
            side: Side::B,
            source: Source::Local(Roots {
                mods: "/mods".into(),
                worlds: "/worlds".into(),
                derived: "/derived".into(),
                archives: "/archives".into(),
            }),
            recorded: 1_700_000_000,
            mod_: Named {
                name: "std1".into(),
                version: 3,
            },
            map: Named {
                name: "demo2".into(),
                version: 7,
            },
        };
        let messages = [
            ToClient::Elapsed(Duration::from_secs(1), Duration::from_secs(60)),
            ToClient::Elapsed(Duration::from_secs(2), Duration::from_secs(60)),
        ];

        let mut writer = Writer::create(&path, &header).unwrap();
        for (i, message) in messages.iter().enumerate() {
            writer
                .write(Duration::from_millis(i as u64 * 10), message)
                .unwrap();
        }
        writer.flush().unwrap();
        drop(writer);

        let reader = Reader::open(&path).unwrap();
        assert_eq!(reader.header().side, Side::B);
        assert_eq!(reader.header().recorded, 1_700_000_000);
        assert_eq!(reader.header().mod_.to_string(), "std1@3");
        assert_eq!(reader.header().map.to_string(), "demo2@7");
        assert!(matches!(
            &reader.header().source,
            Source::Local(roots) if roots.worlds == Path::new("/worlds")
        ));
        let frames: Vec<Frame> = reader.map(Result::unwrap).collect();
        let duration = Reader::duration(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(duration, Duration::from_millis(10));

        assert_eq!(frames.len(), 2);
        assert_eq!(frames[1].elapsed, Duration::from_millis(10));
        assert!(matches!(
            frames[1].message,
            ToClient::Elapsed(elapsed, _) if elapsed == Duration::from_secs(2)
        ));
    }
}
