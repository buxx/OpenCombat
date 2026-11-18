use bevy::asset::io::AssetSourceBuilder;
use bevy::audio::{AudioPlugin, SpatialScale};
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::sprite_render::Wireframe2dPlugin;
use bevy_egui::EguiPlugin;
use bevy_spritesheet_animation::plugin::SpritesheetAnimationPlugin;
use bon::builder;
use oc_root::Wcfg;

use crate::config::{Config, Config_};
#[cfg(feature = "debug")]
use crate::debug;
use crate::ingame::{BattlePhase, InGameState};
use crate::sprites::Animations;
use crate::states::{Game, GameConfig, GuiFiles, PointerIn};
use crate::{
    downloading::DownloadingPlugin, error::ErrorPlugin, fx::FxPlugin, home::HomePlugin,
    ingame::IngamePlugin, network::NetworkPlugin, states::AppState,
};
use crate::{ingame, setup, states, window};

/// Spatial audio uses the distance to attenuate the sound volume. In 2D with the default camera,
/// 1 pixel is 1 unit of distance, so we use a scale so that 100 pixels is 1 unit of distance for
/// audio.
const AUDIO_SCALE: f32 = 1. / 100.0;

pub type InstallFn = Box<dyn Fn(&mut App)>;

#[cfg(feature = "debug")]
use debug::DebugPlugin;

#[builder]
pub fn run(
    config: Config_,
    install: Option<InstallFn>,
    #[builder(default)] logging: bool,
) -> AppExit {
    let mut app = App::new();

    // Mods, worlds and derived files are read from where the roots are (see `oc_root::files`)
    for (name, dir) in config.roots.asset_sources() {
        app.register_asset_source(
            name,
            AssetSourceBuilder::platform_default(&dir.display().to_string(), None),
        );
    }

    let default = DefaultPlugins
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Open Combat".into(),
                resolution: (1280, 900).into(),
                // present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        })
        .set(AudioPlugin {
            default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
            ..default()
        })
        .set(ImagePlugin::default_nearest())
        // Use synchronous to prevent segfault on exit (visible during e2e tests).
        // But can produce render glitch.
        .set(RenderPlugin {
            synchronous_pipeline_compilation: true,
            ..default()
        });

    let default = match logging {
        true => default,
        false => default.disable::<LogPlugin>(),
    };

    app.add_plugins(default)
        .add_plugins(EguiPlugin::default())
        .add_plugins(Wireframe2dPlugin::default())
        .add_plugins(SpritesheetAnimationPlugin)
        .add_plugins(bevy_hanabi::HanabiPlugin)
        .add_plugins(ErrorPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(FxPlugin)
        .add_plugins(HomePlugin)
        .add_plugins(DownloadingPlugin)
        .add_plugins(IngamePlugin)
        .add_plugins(Animations)
        .add_plugins(window::WindowPlugin)
        .add_plugins(ingame::camera::CameraPlugin)
        .insert_state(AppState::Home)
        .init_resource::<Game>()
        .init_resource::<Wcfg>()
        .init_resource::<GameConfig>()
        .init_resource::<GuiFiles>()
        .init_resource::<states::Window>()
        .init_state::<InGameState>()
        .init_state::<BattlePhase>()
        .init_state::<PointerIn>()
        .insert_resource(Config(config))
        .add_systems(Startup, setup::setup);

    if let Some(install) = install {
        install(&mut app);
    }

    #[cfg(feature = "debug")]
    app.add_plugins(DebugPlugin);

    tracing::info!("Start app");
    app.run()
}
