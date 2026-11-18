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
