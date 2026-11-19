use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use directories::UserDirs;
use oc_root::{
    format::{FileError, read_ron, write_ron},
    side::Side,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

const GAMES_PARENT_FOLDER: &str = "Games";
const APP_FOLDER: &str = "Open Combat";
const GAMES_FOLDER: &str = "Games";
const RECORDS_FOLDER: &str = "Records";
const CACHE_FOLDER: &str = "Cache";
const GAME_FILE: &str = "game.ron";
const BATTLE_FOLDER: &str = "Battle";

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unable to determine user home directory")]
    NoHome,
    #[error(transparent)]
    File(#[from] FileError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Game {
    Battle(BattleGame),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleGame {
    pub id: Uuid,
    pub mod_name: String,
    pub battle_name: String,
    pub scenario_name: String,
    pub side: Side,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    #[serde(default)]
    pub status: BattleStatus,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleStatus {
    /// No fight played yet
    #[default]
    Fresh,
    /// Number of fights already played
    InProgress(usize),
    Finished(BattleResult),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleResult {
    Victory,
    Defeat,
}

impl BattleStatus {
    /// Number of fights already played (unknown, so 0, when finished)
    pub fn fights(&self) -> usize {
        match self {
            BattleStatus::Fresh | BattleStatus::Finished(_) => 0,
            BattleStatus::InProgress(fights) => *fights,
        }
    }
}

impl BattleGame {
    pub fn new(
        mod_name: impl Into<String>,
        battle_name: impl Into<String>,
        scenario_name: impl Into<String>,
        side: Side,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            mod_name: mod_name.into(),
            battle_name: battle_name.into(),
            scenario_name: scenario_name.into(),
            side,
            created: now,
            modified: now,
            status: BattleStatus::Fresh,
        }
    }

    /// Update status after a fight from flags owners (`None` when owned by nobody or both
    /// sides): victory when all flags are owned by player side, defeat when all are owned by
    /// opposite side, else one more fight is counted.
    pub fn update_status(&mut self, flags: impl IntoIterator<Item = Option<Side>>) {
        let flags: Vec<_> = flags.into_iter().collect();
        let all_owned_by = |side| !flags.is_empty() && flags.iter().all(|f| *f == Some(side));

        self.status = if all_owned_by(self.side) {
            BattleStatus::Finished(BattleResult::Victory)
        } else if all_owned_by(self.side.opposite()) {
            BattleStatus::Finished(BattleResult::Defeat)
        } else {
            BattleStatus::InProgress(self.status.fights() + 1)
        };
    }
}

/// Application folder in user home. Eg. `/home/username/Games/Open Combat`
pub fn app_dir() -> Result<PathBuf, Error> {
    let dirs = UserDirs::new().ok_or(Error::NoHome)?;
    Ok(dirs.home_dir().join(GAMES_PARENT_FOLDER).join(APP_FOLDER))
}

/// Folder where games are saved. Eg. `/home/username/Games/Open Combat/Games`
pub fn games_dir(app_dir: &Path) -> PathBuf {
    app_dir.join(GAMES_FOLDER)
}

/// Folder where battle records (to replay) are saved. Eg. `/home/username/Games/Open Combat/Records`
pub fn records_dir(app_dir: &Path) -> PathBuf {
    app_dir.join(RECORDS_FOLDER)
}

/// Folder where generated files (like minimaps) are cached. Safe to delete.
pub fn cache_dir(app_dir: &Path) -> PathBuf {
    app_dir.join(CACHE_FOLDER)
}

impl Game {
    pub fn id(&self) -> Uuid {
        match self {
            Game::Battle(game) => game.id,
        }
    }

    /// Game file path.
    /// Example: `<app_dir>/Games/Battle/std1/Demo2/Scenario 1/<uuid>/game.ron`
    pub fn path(&self, app_dir: &Path) -> PathBuf {
        match self {
            Game::Battle(game) => games_dir(app_dir)
                .join(BATTLE_FOLDER)
                .join(folder_name(&game.mod_name))
                .join(folder_name(&game.battle_name))
                .join(folder_name(&game.scenario_name))
                .join(game.id.to_string())
                .join(GAME_FILE),
        }
    }

    /// Game folder
    pub fn dir(&self, app_dir: &Path) -> PathBuf {
        let path = self.path(app_dir);
        path.parent().map(Path::to_path_buf).unwrap_or(path)
    }

    pub fn save(&self, app_dir: &Path) -> Result<PathBuf, Error> {
        let path = self.path(app_dir);
        write_ron(&path, self)?;
        Ok(path)
    }
}

pub fn load_all(app_dir: &Path) -> (Vec<Game>, Vec<String>) {
    let games_dir = games_dir(app_dir);
    let mut games = vec![];
    let mut errors = vec![];

    let files = battle_game_files(&games_dir, &mut errors);

    for path in files {
        match load(&path) {
            Ok(game) => games.push(game),
            Err(error) => errors.push(error.to_string()),
        }
    }

    games.sort_by_key(|Game::Battle(game)| std::cmp::Reverse(game.modified));

    (games, errors)
}

/// Battle game files: `<games_dir>/Battle/<mod>/<battle>/<scenario>/<uuid>/game.ron`.
fn battle_game_files(games_dir: &Path, errors: &mut Vec<String>) -> Vec<PathBuf> {
    let battles_dir = games_dir.join(BATTLE_FOLDER);
    let pattern = format!(
        "{}/*/*/*/*/{GAME_FILE}",
        glob::Pattern::escape(&battles_dir.to_string_lossy())
    );

    match glob::glob(&pattern) {
        Ok(paths) => paths
            .filter_map(|path| path.map_err(|e| errors.push(e.to_string())).ok())
            .collect(),
        Err(error) => {
            errors.push(format!(
                "Search games in '{}': {error}",
                battles_dir.display()
            ));
            vec![]
        }
    }
}

fn load(path: &Path) -> Result<Game, Error> {
    Ok(read_ron(path)?)
}

/// Protect from battle or scenario which own "/" in its name
pub fn folder_name(name: &str) -> String {
    name.replace(['/', '\\'], "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn game() -> Game {
        Game::Battle(BattleGame::new("std1", "Demo2", "Scenario 1", Side::A))
    }

    #[test]
    fn save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let game = game();
        let Game::Battle(inner) = &game;
        let path = game.save(dir.path()).unwrap();

        let expected = format!("Games/Battle/std1/Demo2/Scenario 1/{}/game.ron", inner.id);
        assert_eq!(path, dir.path().join(expected));
        let (games, errors) = load_all(dir.path());
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(games, vec![game]);
    }

    #[test]
    fn multiple_games_for_same_scenario() {
        let dir = tempfile::tempdir().unwrap();
        game().save(dir.path()).unwrap();
        game().save(dir.path()).unwrap();

        let (games, errors) = load_all(dir.path());
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(games.len(), 2);
    }

    #[test]
    fn games_sorted_by_modified_desc() {
        let dir = tempfile::tempdir().unwrap();
        let (mut old, new) = (
            BattleGame::new("std1", "Demo2", "Scenario 1", Side::A),
            game(),
        );
        old.modified -= chrono::Duration::days(1);
        new.save(dir.path()).unwrap();
        Game::Battle(old.clone()).save(dir.path()).unwrap();

        let (games, _) = load_all(dir.path());
        assert_eq!(games.last(), Some(&Game::Battle(old)));
        assert_eq!(games.first(), Some(&new));
    }

    #[test]
    fn update_status() {
        let mut game = BattleGame::new("std1", "Demo2", "Scenario 1", Side::A);
        assert_eq!(game.status, BattleStatus::Fresh);

        game.update_status([Some(Side::A), None, Some(Side::B)]);
        assert_eq!(game.status, BattleStatus::InProgress(1));
        game.update_status([]);
        assert_eq!(game.status, BattleStatus::InProgress(2));

        game.update_status([Some(Side::A), Some(Side::A)]);
        assert_eq!(game.status, BattleStatus::Finished(BattleResult::Victory));

        game.status = BattleStatus::InProgress(3);
        game.update_status([Some(Side::B)]);
        assert_eq!(game.status, BattleStatus::Finished(BattleResult::Defeat));
    }

    #[test]
    fn missing_status_is_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let game = game();
        let path = game.save(dir.path()).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let content: String = content
            .lines()
            .filter(|line| !line.contains("status"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&path, content).unwrap();

        let (games, errors) = load_all(dir.path());
        assert!(errors.is_empty(), "{errors:?}");
        let Game::Battle(loaded) = &games[0];
        assert_eq!(loaded.status, BattleStatus::Fresh);
    }

    #[test]
    fn no_games_folder() {
        let dir = tempfile::tempdir().unwrap();
        let (games, errors) = load_all(&dir.path().join("missing"));
        assert!(games.is_empty() && errors.is_empty());
    }
}
