use bevy::{prelude::*, window::WindowResized};
#[cfg(feature = "debug")]
use oc_geo::tile::{WorldHeightIndex, WorldTileIndex};
use oc_root::geo::WorldVec2;
#[cfg(feature = "debug")]
use oc_utils::bevy::EntityMapping;
#[cfg(feature = "debug")]
use oc_world::{terrain::Terrain, tile::Tile};

use crate::{
    ingame::{
        InGameState, SwitchToWorldMap,
        camera::{
            move_::{MovedBattleCamera, UpdateVisibleBattleSquare},
            region::Region,
        },
        world::{AdjustMinimap, DespawnWorldMapBackground, SpawnWorldMapBackground},
    },
    states::{AppState, PointerIn},
};

#[cfg(feature = "debug")]
use crate::ingame::{
    camera::squad::ShowFormationPositions,
    region::debug::{DespawnRegionWireFrameDebug, SpawnRegionWireFrameDebug},
};

#[cfg(feature = "debug")]
pub mod debug;
pub mod map;
pub mod move_;
pub mod region;
pub mod squad;

pub struct CameraPlugin;

#[derive(Debug, Deref, DerefMut, Clone, Event)]
pub struct GoToPoint(pub WorldVec2);

#[derive(Debug, Default)]
pub enum Focus {
    #[default]
    Battle,
    Height,
    World,
}

#[derive(Debug, Event)]
pub struct WindowResizeWhenWorldMap;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<State>()
            .add_observer(map::on_save_current_window_center_as_battle_center)
            .add_observer(region::on_update_regions)
            .add_observer(move_::on_moved_battle_camera)
            .add_observer(move_::on_go_to_point)
            .add_observer(move_::on_center_camera_on)
            .add_observer(on_window_resize_when_world_map)
            .add_systems(OnEnter(AppState::InGame), init)
            .add_systems(
                Update,
                (update,)
                    .run_if(in_state(AppState::InGame))
                    .after(move_::move_battle),
            )
            .add_systems(
                Update,
                (move_::move_battle,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                Update,
                (move_::move_in_world_map,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::World))
                    .run_if(in_state(PointerIn::Battle)),
            )
            .add_systems(
                Update,
                (watch_window_resize_when_world_map,)
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::World)),
            );

        #[cfg(feature = "debug")]
        app.init_resource::<debug::tile::ShowTiles>()
            .init_resource::<ShowFormationPositions>()
            .init_resource::<EntityMapping<WorldTileIndex>>()
            .init_resource::<EntityMapping<WorldHeightIndex>>()
            .add_observer(debug::tile::on_toggle_show_tiles)
            .add_observer(debug::tile::on_insert_tiles)
            .add_observer(
                debug::tile::on_spawn_region::<
                    debug::tile::SpawnRegionTiles,
                    WorldTileIndex,
                    Tile,
                    Terrain,
                >,
            )
            .add_observer(
                debug::tile::on_spawn_region::<
                    debug::tile::SpawnRegionHeights,
                    WorldHeightIndex,
                    u8,
                    Terrain,
                >,
            )
            .add_observer(debug::tile::on_forgotten_region)
            .add_observer(debug::tile::on_despawn_region_tiles)
            .add_observer(squad::on_toggle_show_formation_positions)
            .add_systems(OnEnter(AppState::InGame), debug::world::setup)
            .add_systems(
                Update,
                debug::world::cursor
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::World)),
            )
            .add_systems(
                Update,
                debug::tile::tile_under_cursor
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Battle)),
            );
    }
}

#[derive(Debug, Default, Resource)]
pub struct State {
    pub cursor: Option<Vec2>,
    pub regions: Option<Vec<Region>>,
    pub focus: Focus, // FIXME: use InGameState instead
    /// Previous camera translation before focus change
    pub previously: Option<Vec3>,
    /// Tile currently under cursor
    #[cfg(feature = "debug")]
    pub tile: Option<WorldTileIndex>,
}

fn init(mut commands: Commands) {
    commands.trigger(MovedBattleCamera)
}

fn update(window: Single<&Window>, mut state: ResMut<State>) {
    state.cursor = window.cursor_position();
}

fn watch_window_resize_when_world_map(
    mut commands: Commands,
    resize_reader: MessageReader<WindowResized>,
) {
    if !resize_reader.is_empty() {
        tracing::debug!("Window resized");
        commands.trigger(WindowResizeWhenWorldMap);
    }
}

fn on_window_resize_when_world_map(
    _: On<WindowResizeWhenWorldMap>,
    mut commands: Commands,
    state: ResMut<State>,
) {
    // Ensure positionning on world elements is done with correct window size
    commands.trigger(SwitchToWorldMap);
    commands.trigger(DespawnWorldMapBackground);
    commands.trigger(SpawnWorldMapBackground);
    commands.trigger(AdjustMinimap);

    if let Some(center) = state.previously {
        commands.trigger(UpdateVisibleBattleSquare(Vec2::new(center.x, center.y)));
    }

    #[cfg(feature = "debug")]
    {
        // Region wireframes on world map need to be sapwn again because depends on window size
        static EMPTY: Vec<Region> = vec![];
        for region in state.regions.as_ref().unwrap_or(&EMPTY) {
            commands.trigger(DespawnRegionWireFrameDebug(region.0));
            commands.trigger(SpawnRegionWireFrameDebug(region.0));
        }
    }
}
