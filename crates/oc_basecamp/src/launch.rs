use oc_root::format::RkyvFile;
use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
};

use std::collections::HashMap;

use oc_battle::SquadPlace;
use oc_deployment::deployment::{Deployment, individual::Status, weapon::Weapons};
use oc_game::Game;
use oc_mod::Mod;
use oc_root::side::Side;
use oc_world::{
    flag::Ownership,
    snapshot::{IndividualState, State},
    spawn::SpawnZoneName,
};
use uuid::Uuid;

use crate::{
    battle::{MAPS_FOLDER, Scenario},
    mods::MODS_FOLDER,
    pages::{PageContext, new_battle},
};

/// Open Combat World State: world state saved by battle at its end
pub const WORLD_STATE_FILE: &str = "world.ocws";
const BATTLE_GUI_BIN: &str = "oc_battle_gui";
const SERVER_CACHE_FOLDER: &str = "server";

/// Deployment file name (in game folder) of the squads engaged in battle by given side
pub fn battle_deployment_file(side: Side) -> &'static str {
    match side {
        Side::A => "a_battle_deployment.yml",
        Side::B => "b_battle_deployment.yml",
    }
}

/// Slots (deployment squad indexes) filled with first not yet taken squads matching `defaults` labels
pub fn default_slots(
    deployment: &Deployment,
    defaults: &[String],
    count: usize,
) -> Vec<Option<usize>> {
    let mut slots = vec![None; count];
    let mut taken = vec![false; deployment.squads.len()];

    for (slot, name) in slots.iter_mut().zip(defaults) {
        let found = deployment
            .squads
            .iter()
            .enumerate()
            .position(|(i, squad)| !taken[i] && squad.label == *name);
        if let Some(index) = found {
            taken[index] = true;
            *slot = Some(index);
        }
    }

    slots
}

pub fn start(ctx: &PageContext) -> Result<Child, String> {
    let game = ctx.current_game.as_ref().ok_or("No current game")?;
    let (battle, scenario) = ctx.selected().ok_or("No scenario selected")?;
    let player = (*ctx.selected_side).ok_or("No side selected")?;
    let dir = game.dir(ctx.app_dir);

    let mut places = vec![];
    for side in [Side::A, Side::B] {
        let slots = if side == player {
            ctx.squad_slots.clone()
        } else {
            let deployment = load_deployment(&dir, side)?;
            let (count, defaults) = side_defaults(scenario, side);
            // FIXME BS NOW: defaults can don't match with available squads: must fill with random
            default_slots(&deployment, defaults, count)
        };

        let deployment = write_battle_deployment(&dir, side, &slots)?;
        places.extend(squad_places(&deployment, &scenario.side(side).places));
    }

    let state = dir.join(WORLD_STATE_FILE);
    let (spawns_a, spawns_b) = battle_spawn_zones(
        &state,
        &scenario.side(Side::A).start_spawn_zones,
        &scenario.side(Side::B).start_spawn_zones,
    )?;
    let spawns = |zones: &[SpawnZoneName]| {
        zones
            .iter()
            .map(|zone| zone.0.as_str())
            .collect::<Vec<_>>()
            .join(",")
    };
    let supports = |side: &str, support: &oc_world::support::Support| {
        support
            .iter()
            .flat_map(|(type_, count)| {
                [
                    format!("--support-{side}"),
                    format!("{}={count}", type_.name()),
                ]
            })
            .collect::<Vec<_>>()
    };
    let order = |side: &str, order: Option<oc_deployment::deployment::place::OrderPolicy>| {
        order
            .map(|order| vec![format!("--order-{side}"), order.arg_name().to_string()])
            .unwrap_or_default()
    };
    let side = match player {
        Side::A => "a",
        Side::B => "b",
    };
    let cwd = std::env::current_dir().map_err(|e| format!("Current folder: {e}"))?;
    let cache = oc_game::cache_dir(ctx.app_dir).join(SERVER_CACHE_FOLDER);

    let mut command = Command::new(battle_gui_bin());
    command
        .args(["--side", side])
        .args(ctx.battle_args)
        .arg("embedded")
        .arg(Path::new(MODS_FOLDER).join(ctx.selected_mod))
        .arg(Path::new(MAPS_FOLDER).join(battle.map()))
        .arg("--deployment-a")
        .arg(dir.join(battle_deployment_file(Side::A)))
        .arg("--deployment-b")
        .arg(dir.join(battle_deployment_file(Side::B)))
        .args(["--spawns-a", &spawns(&spawns_a)])
        .args(["--spawns-b", &spawns(&spawns_b)])
        .args(supports("a", &scenario.side(Side::A).supports))
        .args(supports("b", &scenario.side(Side::B).supports))
        .args(order("a", scenario.side(Side::A).order))
        .args(order("b", scenario.side(Side::B).order))
        .args(
            places
                .iter()
                .flat_map(|(uuid, place)| ["--place".to_string(), format!("{uuid}={place}")]),
        )
        .arg("--cache")
        .arg(cache)
        .arg(format!("--end=save-map-persist-to:{}", state.display()))
        // Gui assets are in the `assets` folder of current folder (like mods, maps, etc.)
        .env("BEVY_ASSET_ROOT", cwd);

    command
        .spawn()
        .map_err(|e| format!("Start {}: {e}", command.get_program().display()))
}

/// Spawn zones owned by side A and B at battle start: from world state saved by previous battle
/// (`state` file) when it exists, elsewhere scenario start spawn zones.
fn battle_spawn_zones(
    state: &Path,
    a_start: &[SpawnZoneName],
    b_start: &[SpawnZoneName],
) -> Result<(Vec<SpawnZoneName>, Vec<SpawnZoneName>), String> {
    if !state.is_file() {
        return Ok((a_start.to_vec(), b_start.to_vec()));
    }

    let state =
        State::load(state).map_err(|e| format!("Load world state '{}': {e}", state.display()))?;
    Ok((state.spawns.a, state.spawns.b))
}

/// Battle gui binary: next to current executable, elsewhere from PATH
#[cfg(not(feature = "debug"))]
fn battle_gui_bin() -> PathBuf {
    let name = format!("{BATTLE_GUI_BIN}{}", std::env::consts::EXE_SUFFIX);
    std::env::current_exe()
        .ok()
        .map(|exe| exe.with_file_name(&name))
        .filter(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from(name))
}

/// Debug battle gui binary: searched in `target/debug` of current executable, current folder
/// or sources ancestors
#[cfg(feature = "debug")]
fn battle_gui_bin() -> PathBuf {
    let name = format!("{BATTLE_GUI_BIN}{}", std::env::consts::EXE_SUFFIX);
    let starts = [
        std::env::current_exe().ok(),
        std::env::current_dir().ok(),
        Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))),
    ];

    starts
        .iter()
        .flatten()
        .flat_map(|start| start.ancestors())
        .flat_map(|dir| [dir.join("target"), dir.to_path_buf()])
        .map(|target| target.join("debug").join(&name))
        .find(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}

fn side_defaults(scenario: &Scenario, side: Side) -> (usize, &[String]) {
    let side = scenario.side(side);
    (side.squad_count, &side.default)
}

fn squad_places(deployment: &Deployment, places: &[SquadPlace]) -> Vec<(Uuid, String)> {
    let mut taken = vec![false; deployment.squads.len()];

    places
        .iter()
        .filter_map(|place| {
            let index = deployment
                .squads
                .iter()
                .enumerate()
                .position(|(i, squad)| !taken[i] && squad.label == place.squad())?;
            taken[index] = true;
            Some((deployment.squads[index].uuid, place.place().to_string()))
        })
        .collect()
}

/// Side deployment (all available squads) from game folder
pub fn load_deployment(dir: &Path, side: Side) -> Result<Deployment, String> {
    let path = dir.join(new_battle::deployment_file(side));
    Deployment::from_file(&path).map_err(|e| format!("{e:#}"))
}

/// Update side A and B deployments files and game status with world state
pub fn apply_world_state(app_dir: &Path, game: &mut Game, state: &State) -> Result<(), String> {
    let dir = game.dir(app_dir);
    let Game::Battle(battle) = game;
    let mod_ = Mod::load(&Path::new(MODS_FOLDER).join(&battle.mod_name), None)
        .map_err(|e| format!("Load mod '{}': {e}", battle.mod_name))?;
    let states: HashMap<_, &IndividualState> =
        state.individuals.iter().map(|i| (i.uuid, i)).collect();

    for side in [Side::A, Side::B] {
        let mut deployment = load_deployment(&dir, side)?;

        for squad in &mut deployment.squads {
            for individual in &mut squad.individuals {
                if let Some(state) = states.get(&individual.uuid) {
                    individual.status = Status::from(&state.status);
                    individual.weapons = weapons(&mod_, &state.weapons);
                }
            }
            squad
                .individuals
                .retain(|individual| individual.status != Status::Dead);
        }
        // Squads where all individuals are dead
        deployment
            .squads
            .retain(|squad| !squad.individuals.is_empty());

        write_deployment(&dir.join(new_battle::deployment_file(side)), &deployment)?;
    }

    battle.update_status(state.flags.iter().map(|(_, ownership)| match ownership {
        Ownership::A => Some(Side::A),
        Ownership::B => Some(Side::B),
        Ownership::Nobody | Ownership::Both => None,
    }));

    Ok(())
}

/// Deployment (mod names) form of world individual weapons (mod indexes)
fn weapons(mod_: &Mod, weapons: &oc_individual::Weapons) -> Weapons {
    Weapons {
        main: weapons
            .primary
            .as_ref()
            .map(|weapon| mod_.weapon(weapon.i).name().to_string()),
        grenades: weapons
            .grenades
            .iter()
            .map(|grenade| mod_.grenade(*grenade).name.clone())
            .collect(),
    }
}

fn write_deployment(path: &Path, deployment: &Deployment) -> Result<(), String> {
    oc_root::format::write_yaml(path, deployment).map_err(|e| e.to_string())
}

fn write_battle_deployment(
    dir: &Path,
    side: Side,
    slots: &[Option<usize>],
) -> Result<Deployment, String> {
    let deployment = load_deployment(dir, side)?;
    let mut squads: Vec<_> = deployment.squads.into_iter().map(Some).collect();
    let squads = slots
        .iter()
        .flatten()
        .filter_map(|index| squads.get_mut(*index).and_then(Option::take))
        .collect();
    let deployment = Deployment {
        squads,
        spawns: vec![],
    };

    write_deployment(&dir.join(battle_deployment_file(side)), &deployment)?;
    Ok(deployment)
}

#[cfg(test)]
mod tests {
    use oc_root::side::BySide;
    use oc_world::flag::Flags;

    use super::*;

    fn names(values: &[&str]) -> Vec<SpawnZoneName> {
        values.iter().map(|v| SpawnZoneName::new(*v)).collect()
    }

    fn squad(label: &str) -> oc_deployment::deployment::squad::Squad {
        oc_deployment::deployment::squad::Squad {
            uuid: Uuid::new_v4(),
            label: label.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn squad_places_by_label() {
        let deployment = Deployment {
            squads: vec![squad("Riffle"), squad("Mg34"), squad("Riffle")],
            spawns: vec![],
        };
        let place = |squad: &str, place: &str| SquadPlace(squad.into(), place.into());
        let places = [
            place("Riffle", "1"),
            place("Riffle", "2"),
            place("Riffle", "3"),
            place("Bren", "4"),
            place("Mg34", "5"),
        ];

        let placed = squad_places(&deployment, &places);

        let uuid = |i: usize| deployment.squads[i].uuid;
        assert_eq!(
            placed,
            vec![
                (uuid(0), "1".to_string()),
                (uuid(2), "2".to_string()),
                (uuid(1), "5".to_string()),
            ]
        );
    }

    #[test]
    fn first_battle_uses_scenario_start_spawn_zones() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join(WORLD_STATE_FILE);

        let spawns = battle_spawn_zones(&state, &names(&["A1"]), &names(&["B1", "B2"]));

        assert_eq!(spawns, Ok((names(&["A1"]), names(&["B1", "B2"]))));
    }

    #[test]
    fn next_battle_uses_world_state_spawn_zones() {
        let dir = tempfile::tempdir().unwrap();
        let state = dir.path().join(WORLD_STATE_FILE);
        State::new(
            vec![],
            vec![],
            vec![],
            BySide::new(names(&["A1", "B1"]), names(&["B2"])),
            vec![],
            Flags::default(),
        )
        .save(&state)
        .unwrap();

        let spawns = battle_spawn_zones(&state, &names(&["A1"]), &names(&["B1", "B2"]));

        assert_eq!(spawns, Ok((names(&["A1", "B1"]), names(&["B2"]))));
    }
}
