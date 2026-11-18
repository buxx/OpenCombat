mod about;
pub mod app;
mod battle;
mod errors;
mod game;
mod launch;
mod map;
mod map_view;
mod minimap;
mod mods;
pub mod page;
mod pages;

pub const UI_ZOOM: f32 = 1.25;

/// Data folders (mods, maps, battles) are read relative to the current folder. When they are
/// next to the executable (release archive), use the executable folder as current folder so the
/// game works wherever it is started from. Otherwise (dev, `cargo run`), keep current folder.
pub fn use_exe_dir_if_data_beside() -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    let Some(dir) = exe.parent() else {
        return Ok(());
    };
    let data_beside = [mods::MODS_FOLDER, battle::MAPS_FOLDER, battle::BATTLES_FOLDER]
        .iter()
        .all(|folder| dir.join(folder).is_dir());
    if data_beside {
        std::env::set_current_dir(dir)?;
    }
    Ok(())
}
