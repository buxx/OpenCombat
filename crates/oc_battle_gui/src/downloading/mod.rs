use crate::states::GuiFiles;
use bevy::prelude::*;
use oc_root::files::{self, File, Files};
use oc_utils::let_some;

use crate::{
    http_to_file, ingame,
    sprites::decor::DecorSprites,
    states::{AppState, GameConfig, PointerIn},
    utils::untar,
    world::World,
};

#[derive(Event)]
pub struct Downloaded;

pub struct DownloadingPlugin;

impl Plugin for DownloadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_downloaded)
            .add_systems(Update, download.run_if(in_state(AppState::Downloading)));
    }
}

// TODO: for now, this action is blocking, it should be not and display a progress message
// TODO: refact this ugly non refactored download / cache / assets code
#[allow(clippy::too_many_arguments)]
fn download(
    mut commands: Commands,
    g: Res<GameConfig>,
    files: Res<GuiFiles>,
    mut world_: ResMut<World>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) -> Result<(), BevyError> {
    let_some!(g = &g.0, return Ok(()));
    let_some!(files = &files.0, return Ok(()));

    tracing::info!("Download");
    let r_width = g.w.region_width();
    let r_height = g.w.region_height();

    ensure_file(files, File::ModArchive).unwrap(); // TODO
    ensure_file(files, File::WorldArchive).unwrap(); // TODO
    ensure_file(files, File::IndividualOutline).unwrap(); // TODO

    for index in 0..g.w.regions_count() {
        let region = File::Region {
            index,
            width: r_width,
            height: r_height,
        };
        ensure_file(files, region).unwrap(); // TODO
    }

    let map = oc_world::reader::MapReader::new(&files.world_dir()).unwrap(); // TODO
    for interior in map.interiors().unwrap() {
        ensure_file(files, File::Interior(interior.id)).unwrap(); // TODO
        // TODO: when quit ingame state, must despawn
        ingame::interior::spawn(&mut commands, &assets, files, &g.w, interior.clone());
    }

    // TODO unwrap
    for spawn_zone in map.spawn_zones().unwrap() {
        // TODO: when quit ingame state, must despawn
        ingame::spawn::spawn_zone(&mut commands, &mut meshes, &mut materials, &g.w, spawn_zone);
    }

    // FIXME: check tile size
    let terrain =
        oc_world::terrain::Terrain::load(&files.terrain_tsx(), g.w.clone(), &g.mod_).unwrap(); // TODO
    tracing::trace!(name="downloading-terrain", terrain=?terrain);
    world_.terrain = Some(terrain); // TODO

    let decor_tileset = oc_world::decor::DecorTileset::load(&files.decor_tsx()).unwrap(); // TODO
    commands.insert_resource(DecorSprites::init(
        &files.decor_png(),
        decor_tileset,
        &assets,
        &mut atlas_layouts,
    ));

    commands.trigger(Downloaded);

    Ok(())
}

pub fn ensure_file(files: &Files, file: File) -> Result<(), Box<dyn std::error::Error>> {
    let Some((sync, target)) = files.sync(file) else {
        tracing::info!("Use local {file}");
        return Ok(());
    };

    if target.exists() {
        tracing::debug!("File {} already exists", target.display());
        return Ok(());
    }

    tracing::info!("Download {file}");
    tracing::debug!("Ensure file with sync {sync} ({})", target.display());

    let mut partial = target.clone().into_os_string();
    partial.push(".part");
    let partial = std::path::PathBuf::from(partial);
    if partial.is_dir() {
        std::fs::remove_dir_all(&partial)?;
    } else if partial.exists() {
        std::fs::remove_file(&partial)?;
    }

    match sync {
        files::Sync::DirectDownload(url) => {
            tracing::debug!("Direct download from {url}");
            http_to_file!(url, &partial);
        }
        files::Sync::ArchiveDownload(url) => {
            tracing::debug!("Download archive from {url}");
            let (_, path) = tempfile::NamedTempFile::new()?.keep()?;
            http_to_file!(url, &path);
            std::fs::create_dir_all(&partial)?;
            untar(&path, &partial)?;
        }
    }

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::rename(&partial, &target)?;

    Ok(())
}

fn on_downloaded(
    _: On<Downloaded>,
    mut app_state: ResMut<NextState<AppState>>,
    mut pointer: ResMut<NextState<PointerIn>>,
) {
    tracing::info!("Download finished");
    tracing::debug!("Entering 'Ingame' state");
    *app_state = NextState::Pending(AppState::InGame);
    *pointer = NextState::Pending(PointerIn::Battle)
}
