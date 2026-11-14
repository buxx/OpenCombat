//! End-to-end: start a battle from basecamp, battle gui ends it by itself and exits.
//!
//! Needs a display and the debug battle gui (searched in `target/debug`) built first.
//! Run with `just test-basecamp-battle-test1` (or as part of `just test-e2e`).
#![cfg(feature = "e2e")]

mod common;

use std::time::{Duration, Instant};

use eframe::egui::accesskit::Role;
use egui_kittest::{Harness, kittest::Queryable};
use oc_basecamp::{app::App, page::Page};

/// Maximum time for the battle gui to start, end the battle and exit
const BATTLE_TIMEOUT: Duration = Duration::from_secs(300);
const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[test]
fn solo_battle_test1_starts_and_exits() {
    // Battle gui ends the battle as soon as it receives the game config
    let battle_args = vec!["--debug-auto-end".to_string()];
    let mut basecamp = common::start_with(move |app| app.with_battle_args(battle_args));
    let harness = &mut basecamp.harness;

    select_mod(harness, "tests1");
    click_through(
        harness,
        &[
            ("Solo", Page::Solo),
            ("Battle", Page::Battle),
            ("Test1", Page::Battle),
            ("Scenario 1", Page::Battle),
            ("New game", Page::NewBattle),
            ("Start", Page::PrepareBattle),
            ("Choose Allied", Page::ChooseSquads),
            ("Start battle", Page::BattleInProgress),
        ],
    );
    wait_battle_end(harness);
}

#[test]
fn solo_battle_test1_killed_squad_is_no_more_available() {
    // Battle gui kills first squad of side A (the first slot: "Riffle"), then ends the battle
    let battle_args = ["--debug-auto-end", "--debug-kill-squad", "a:0"]
        .map(String::from)
        .to_vec();
    let mut basecamp = common::start_with(move |app| app.with_battle_args(battle_args));
    let harness = &mut basecamp.harness;

    select_mod(harness, "tests1");
    click_through(
        harness,
        &[
            ("Solo", Page::Solo),
            ("Battle", Page::Battle),
            ("Test1", Page::Battle),
            ("Scenario 2", Page::Battle),
            ("New game", Page::NewBattle),
            ("Start", Page::PrepareBattle),
            ("Choose Allied", Page::ChooseSquads),
        ],
    );

    // Both squads available (and placed in default slots), with their soldiers
    assert_squads(harness, &[("Riffle", "oo"), ("Assault", "ooo")]);

    click_through(harness, &[("Start battle", Page::BattleInProgress)]);
    wait_battle_end(harness);

    // Back on choose squads page: killed squad is no more available
    assert_only_assault(harness);

    // Same when the game is reopened from the scenario games list
    click_through(harness, &[("Battle", Page::Battle)]);
    harness.get_by_label_contains("(Allied)").click();
    harness.run_ok();
    assert_eq!(
        harness.state().page(),
        Page::ChooseSquads,
        "after opening game (errors: {:?})",
        harness.state().error_messages()
    );
    assert_only_assault(harness);
}

#[test]
fn solo_battle_test1_all_flags_owned_is_finished() {
    // Battle gui gives all flags to side A (Allied, the player side), then ends the battle
    let battle_args = ["--debug-auto-end", "--debug-own-flags", "a"]
        .map(String::from)
        .to_vec();
    let mut basecamp = common::start_with(move |app| app.with_battle_args(battle_args));
    let harness = &mut basecamp.harness;

    select_mod(harness, "tests1");
    click_through(
        harness,
        &[
            ("Solo", Page::Solo),
            ("Battle", Page::Battle),
            ("Test1", Page::Battle),
            ("Scenario 1", Page::Battle),
            ("New game", Page::NewBattle),
            ("Start", Page::PrepareBattle),
            ("Choose Allied", Page::ChooseSquads),
            ("Start battle", Page::BattleInProgress),
        ],
    );
    wait_battle_end_on(harness, Page::BattleFinished);
    harness.get_by_label("Victory");

    // Same when the game is reopened from the scenario games list
    click_through(harness, &[("Battle", Page::Battle)]);
    harness.get_by_label_contains("(Allied)").click();
    harness.run_ok();
    assert_eq!(
        harness.state().page(),
        Page::BattleFinished,
        "after opening game (errors: {:?})",
        harness.state().error_messages()
    );
    harness.get_by_label("Victory");
}

fn assert_only_assault(harness: &Harness<'static, App>) {
    assert_squads(harness, &[("Assault", "ooo")]);
    assert!(
        harness.query_by_label("Riffle").is_none(),
        "killed squad 'Riffle' still shown"
    );
    assert!(harness.query_by_label("oo").is_none());
}

fn select_mod(harness: &mut Harness<'static, App>, name: &str) {
    harness.get_by_role(Role::ComboBox).click();
    harness.run_ok();
    harness.get_by_label(name).click();
    harness.run_ok();
}

fn click_through(harness: &mut Harness<'static, App>, steps: &[(&str, Page)]) {
    for (label, page) in steps {
        harness.get_by_label(label).click();
        harness.run_ok();
        assert_eq!(
            harness.state().page(),
            *page,
            "after clicking '{label}' (errors: {:?})",
            harness.state().error_messages()
        );
    }
}

/// Each squad is shown in squads list and in its (default) slot, with one "o" by soldier
fn assert_squads(harness: &Harness<'static, App>, squads: &[(&str, &str)]) {
    for (label, soldiers) in squads {
        assert_eq!(
            harness.query_all_by_label(label).count(),
            2,
            "squad '{label}' must be shown in list and slot"
        );
        assert_eq!(
            harness.query_all_by_label(soldiers).count(),
            2,
            "squad '{label}' soldiers '{soldiers}' must be shown in list and slot"
        );
    }
}

/// Wait battle gui ends the battle and exits, then basecamp shows choose squads page
fn wait_battle_end(harness: &mut Harness<'static, App>) {
    wait_battle_end_on(harness, Page::ChooseSquads);
}

/// Wait battle gui ends the battle and exits, then basecamp shows `page`
fn wait_battle_end_on(harness: &mut Harness<'static, App>, page: Page) {
    let started = Instant::now();
    while harness.state().page() == Page::BattleInProgress {
        assert!(
            started.elapsed() < BATTLE_TIMEOUT,
            "battle gui did not exit within {BATTLE_TIMEOUT:?}"
        );
        std::thread::sleep(POLL_INTERVAL);
        harness.step();
    }
    harness.run_ok();

    // Successful exit goes to choose squads page (or battle finished page, when battle is
    // finished), with deployments and game status updated from world state
    assert_eq!(
        harness.state().error_messages(),
        &[] as &[String],
        "battle errors (is target/debug/oc_battle_gui built with `--features debug`?)"
    );
    assert_eq!(harness.state().page(), page);
}
