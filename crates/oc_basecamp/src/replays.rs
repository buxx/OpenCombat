use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
};

use chrono::{DateTime, Local};
use oc_replay::{Header, Reader};

use crate::launch;

pub const RECORD_EXTENSION: &str = "ocrec";

#[derive(Debug, Clone)]
pub struct Record {
    pub path: PathBuf,
    pub header: Header,
}

impl Record {
    pub fn datetime(&self) -> DateTime<Local> {
        DateTime::from_timestamp(self.header.recorded as i64, 0)
            .unwrap_or_default()
            .with_timezone(&Local)
    }

    /// Name as eg. `std1@1 demo2@1 2026-11-19 14:03:22`
    pub fn name(&self) -> String {
        format!(
            "{} {} {}",
            self.header.mod_,
            self.header.map,
            self.datetime().format("%Y-%m-%d %H:%M:%S")
        )
    }
}

pub fn new_record_path(app_dir: &Path) -> Result<PathBuf, String> {
    let dir = oc_game::records_dir(app_dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Create '{}': {e}", dir.display()))?;
    let name = Local::now().format("%Y-%m-%d_%H-%M-%S");
    Ok(dir.join(format!("{name}.{RECORD_EXTENSION}")))
}

/// Records of the records folder, most recent first
pub fn load_all(app_dir: &Path) -> (Vec<Record>, Vec<String>) {
    let dir = oc_game::records_dir(app_dir);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        // No record yet
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return (vec![], vec![]),
        Err(error) => return (vec![], vec![format!("Read '{}': {error}", dir.display())]),
    };

    let mut records = vec![];
    let mut errors = vec![];

    for path in entries.filter_map(Result::ok).map(|e| e.path()) {
        if path.extension().is_none_or(|e| e != RECORD_EXTENSION) {
            continue;
        }
        match Reader::read_header(&path) {
            Ok(header) => records.push(Record { path, header }),
            Err(error) => errors.push(format!("Record '{}': {error}", path.display())),
        }
    }

    records.sort_by_key(|record| std::cmp::Reverse(record.header.recorded));
    (records, errors)
}

/// Start battle gui play `record`
pub fn play(record: &Record, battle_args: &[String]) -> Result<Child, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("Current folder: {e}"))?;
    let mut command = Command::new(launch::battle_gui_bin());
    command
        .args(battle_args)
        .arg("replay")
        .arg(&record.path)
        // Gui assets are in the `assets` folder of current folder
        .env("BEVY_ASSET_ROOT", cwd);

    command
        .spawn()
        .map_err(|e| format!("Start {}: {e}", command.get_program().display()))
}
