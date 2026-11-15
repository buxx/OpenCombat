//! Game lifecycle: creation (when player choose its side), opening, update after a battle
use oc_root::format::RkyvFile;
use std::path::{Path, PathBuf};

use oc_deployment::generator::Generator;
use oc_game::{BattleGame, Game};
use oc_mod::Mod;
use oc_root::side::Side;
use oc_world::snapshot::State;

use crate::{
    battle::{Battle, Scenario},
    launch::{self, WORLD_STATE_FILE},
    minimap,
    mods::{MODS_FOLDER, ModInfo},
    pages::{PageContext, new_battle},
};

/// Game minimap (spawn zones owned by each side), in game folder
const MINIMAP_FILE: &str = "minimap.png";

pub fn minimap_path(app_dir: &Path, game: &Game) -> PathBuf {
    game.dir(app_dir).join(MINIMAP_FILE)
}

/// Create the game (played with `side`) with its side A and B deployments files and its minimap
pub fn create(
    app_dir: &Path,
    battle: &Battle,
    scenario: &Scenario,
    mod_: &ModInfo,
    side: Side,
) -> Result<Game, String> {
    let mod_name = &mod_.name;
    let loaded = Mod::load(&Path::new(MODS_FOLDER).join(mod_name), None)
        .map_err(|e| format!("Load mod '{mod_name}': {e}"))?;

    let generator = Generator::new();
    let mut deployments = vec![];
    for (side, profile) in [
        (Side::A, scenario.deploy_a()),
        (Side::B, scenario.deploy_b()),
    ] {
        let file = new_battle::deployment_file(side);
        let deployment = generator
            .generate(&loaded, profile)
            .map_err(|e| format!("Generate {file}: {e}"))?;
        let yaml = serde_yaml::to_string(&deployment).map_err(|e| format!("{file}: {e}"))?;
        deployments.push((file, yaml));
    }

    let game = Game::Battle(BattleGame::new(
        mod_name,
        battle.name(),
        scenario.name(),
        side,
    ));
    let game_path = game.save(app_dir).map_err(|e| e.to_string())?;
    let dir = game_path.parent().ok_or("Invalid game path")?;
    let files = write_deployments(dir, deployments).and_then(|_| {
        minimap::generate_with_zones(
            app_dir,
            battle,
            mod_,
            scenario.a_start_spawn_zones(),
            scenario.b_start_spawn_zones(),
            &minimap_path(app_dir, &game),
        )
    });
    if let Err(error) = files {
        // Do not leave an incomplete game (folder was just created for this game)
        let _ = std::fs::remove_dir_all(dir);
        return Err(error);
    }

    Ok(game)
}

fn write_deployments(dir: &Path, deployments: Vec<(&str, String)>) -> Result<(), String> {
    for (file, yaml) in deployments {
        let path = dir.join(file);
        std::fs::write(&path, yaml).map_err(|e| format!("Write '{}': {e}", path.display()))?;
    }
    Ok(())
}

/// Make `game` the current game: load its player side deployment and fill default slots
pub fn open(ctx: &mut PageContext, scenario: &Scenario, game: Game) -> Result<(), String> {
    let Game::Battle(battle_game) = &game;
    let side = battle_game.side;
    let deployment = launch::load_deployment(&game.dir(ctx.app_dir), side)?;

    let (count, defaults) = match side {
        Side::A => (scenario.a_squad_count(), scenario.a_default()),
        Side::B => (scenario.b_squad_count(), scenario.b_default()),
    };
    *ctx.squad_slots = launch::default_slots(&deployment, defaults, count);
    *ctx.deployment = Some(deployment);
    *ctx.selected_side = Some(side);
    *ctx.current_game = Some(game);

    Ok(())
}

/// Update game files with the world state saved by battle: deployments, minimap and game
/// modification date.
pub fn after_battle(
    app_dir: &Path,
    game: &mut Game,
    battle: &Battle,
    mod_: &ModInfo,
) -> Result<(), String> {
    let state =
        State::load(&game.dir(app_dir).join(WORLD_STATE_FILE)).map_err(|e| e.to_string())?;

    launch::apply_world_state(app_dir, game, &state)?;
    minimap::generate_with_zones(
        app_dir,
        battle,
        mod_,
        &state.side_a_spawns,
        &state.side_b_spawns,
        &minimap_path(app_dir, game),
    )?;

    let Game::Battle(battle_game) = game;
    battle_game.modified = chrono::Utc::now();
    game.save(app_dir).map_err(|e| e.to_string())?;

    Ok(())
}
