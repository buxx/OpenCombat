mod common;

use egui_kittest::kittest::Queryable;
use oc_basecamp::page::Page;

#[test]
fn starts_on_home_without_errors() {
    let basecamp = common::start();

    assert_eq!(basecamp.harness.state().page(), Page::Home);
    assert_eq!(basecamp.harness.state().error_messages(), &[] as &[String]);
}

#[test]
fn navigate_to_battle_and_back_home() {
    let mut basecamp = common::start();
    let harness = &mut basecamp.harness;

    harness.get_by_label("Solo").click();
    harness.run();
    assert_eq!(harness.state().page(), Page::Solo);

    harness.get_by_label("Battle").click();
    harness.run();
    assert_eq!(harness.state().page(), Page::Battle);

    // Breadcrumb link
    harness.get_by_label("Home").click();
    harness.run();
    assert_eq!(harness.state().page(), Page::Home);
}

#[test]
fn replays_page_lists_records() {
    let mut basecamp = common::start();
    let records = basecamp._app_dir.path().join("Records");
    std::fs::create_dir_all(&records).unwrap();
    let header = oc_replay::Header {
        side: oc_root::side::Side::A,
        source: oc_replay::Source::Server("127.0.0.1:6589".parse().unwrap()),
        recorded: 1_700_000_000,
        mod_: oc_replay::Named {
            name: "std1".into(),
            version: 1,
        },
        map: oc_replay::Named {
            name: "demo2".into(),
            version: 3,
        },
    };
    oc_replay::Writer::create(&records.join("a.ocrec"), &header)
        .unwrap()
        .flush()
        .unwrap();
    // Not a record: ignored
    std::fs::write(records.join("notes.txt"), "").unwrap();
    let harness = &mut basecamp.harness;

    harness.get_by_label("Replays").click();
    harness.run();
    assert_eq!(harness.state().page(), Page::Replays);

    let date = chrono::DateTime::from_timestamp(1_700_000_000, 0)
        .unwrap()
        .with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S");
    harness.get_by_label(&format!("std1@1 demo2@3 {date}"));
    assert_eq!(harness.state().error_messages(), &[] as &[String]);
}
