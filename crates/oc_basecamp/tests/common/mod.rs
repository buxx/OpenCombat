use std::sync::Once;

use egui_kittest::Harness;
use oc_basecamp::{UI_ZOOM, app::App};
use tempfile::TempDir;

static WORKSPACE_CWD: Once = Once::new();

/// Basecamp reads `mods`, `battles` and `maps` relative to the current directory
/// (the workspace root when launched normally). Cargo runs integration tests from
/// the crate folder, so move to the workspace root once for the whole test binary.
fn enter_workspace_root() {
    WORKSPACE_CWD.call_once(|| {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
        std::env::set_current_dir(root).expect("enter workspace root");
    });
}

/// A running basecamp with an isolated application folder (games, cache).
/// Keep `_app_dir` alive for the whole test: dropping it deletes the folder.
pub struct Basecamp {
    pub harness: Harness<'static, App>,
    pub _app_dir: TempDir,
}

/// Start basecamp and run frames until the initial loading is done.
#[allow(dead_code)] // Not used by every test binary
pub fn start() -> Basecamp {
    start_with(|app| app)
}

/// Like [`start`], with `configure` applied to the application before it runs.
pub fn start_with(configure: impl FnOnce(App) -> App + 'static) -> Basecamp {
    enter_workspace_root();

    let app_dir = TempDir::new().expect("create temporary app dir");
    let path = app_dir.path().to_path_buf();

    let mut harness = Harness::builder()
        .with_size(eframe::egui::vec2(1600.0, 1000.0))
        .build_eframe(move |cc| {
            cc.egui_ctx.set_zoom_factor(UI_ZOOM);
            egui_extras::install_image_loaders(&cc.egui_ctx);
            configure(App::with_app_dir(path))
        });
    harness.run();
    assert!(!harness.state().is_loading(), "basecamp still loading");

    Basecamp {
        harness,
        _app_dir: app_dir,
    }
}
