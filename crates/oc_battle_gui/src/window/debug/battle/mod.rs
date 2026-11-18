use std::fmt::Display;

use bevy::prelude::*;
use derive_more::Constructor;
use oc_geo::{region::RegionXy, tile::TileXy};
use oc_individual::IndividualIndex;
use oc_mod::{
    DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS, Mod,
    ammunition::IndexedAmmunition,
    armament::IndexedShotMode,
    weapons::{IndexedWeapon, WeaponType},
};
use oc_projectile::ProjectileId;
use oc_root::{WorldConfig, physics::Meters, side::Side};
use strum_macros::EnumIter;

pub mod component;
pub mod cursor;
pub mod left_click;
pub mod mixing;
pub mod refresh;
pub mod states;
pub mod weapons;
pub mod window;

use crate::{
    ingame::{
        self,
        camera::region::Region,
        input::left_click::LeftClickModeType,
        lov::{LovClickMode, SpawnProjectileClickMode},
    },
    window::{
        MountedWindow, UnmountedWindow,
        debug::{battle::component::ComponentDetails, subject::Subject},
    },
};

/// Used to cache debug battle window when not displayed
#[derive(Resource, Deref, DerefMut, Default)]
pub struct DebugBattleWindow(pub Option<window::Window>);

#[derive(Debug, Event)]
pub struct Refresh;

#[derive(Debug, Clone)]
pub struct Context {
    refresh: refresh::Refresh,
    show_tiles: bool,
    show_formation_positions: bool,
    show_visibilities: bool,
    // Cursor
    cursor: Option<Vec2>,
    point: Option<Vec2>,
    tile: Option<(TileXy, String)>,
    region: Option<RegionXy>,
    // Components
    component_view: component::View,
    component_details: Option<ComponentDetails>,
    states_view: states::View,
    regions: Vec<Region>,
    individuals: Vec<Subject<IndividualIndex>>,
    projectiles: Vec<Subject<ProjectileId>>,
    ingame: ingame::state::State,
    // Left click
    left_click_mode: LeftClickModeType,
    spawn_weapon_type: WeaponType,
    spawn_weapon: Option<IndexedWeapon>,
    spawn_ammunition: Option<IndexedAmmunition>,
    spawn_shot: Option<IndexedShotMode>,
    spawn_projectile_click_mode: SpawnProjectileClickMode,
    spawn_repeat: u8,
    spawn_projectile_plus_z: Meters,
    spawn_projectile_side: Side,
    explosion_shell: Option<oc_mod::shell::ShellIndex>,
    master_volume: f32,
    bus_volumes: Vec<(oc_physics::fx::SoundBus, f32)>,
    lov_click_mode: LovClickMode,
    lov_start_plus_z: Meters,
    lov_end_plus_z: Meters,
}

impl Default for Context {
    fn default() -> Self {
        let volumes = crate::fx::BusVolumes::default();
        let bus_volumes = oc_physics::fx::SoundBus::ALL
            .iter()
            .map(|bus| (*bus, volumes.linear(*bus)))
            .collect();

        Self {
            refresh: Default::default(),
            show_tiles: Default::default(),
            show_formation_positions: Default::default(),
            show_visibilities: Default::default(),
            cursor: Default::default(),
            point: Default::default(),
            tile: Default::default(),
            region: Default::default(),
            component_view: Default::default(),
            component_details: Default::default(),
            states_view: Default::default(),
            regions: Default::default(),
            individuals: Default::default(),
            projectiles: Default::default(),
            ingame: Default::default(),
            left_click_mode: LeftClickModeType::Select,
            spawn_weapon_type: Default::default(),
            spawn_weapon: Default::default(),
            spawn_ammunition: Default::default(),
            spawn_shot: Default::default(),
            spawn_projectile_click_mode: Default::default(),
            spawn_repeat: 1,
            spawn_projectile_plus_z: DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS,
            spawn_projectile_side: Side::A,
            explosion_shell: Default::default(),
            master_volume: crate::fx::BusVolumes::default().master,
            bus_volumes,
            lov_click_mode: Default::default(),
            lov_start_plus_z: DEFAULT_HUMAN_DEFAULT_STAND_UP_FIRE_METERS,
            lov_end_plus_z: Meters(0.),
        }
    }
}

#[derive(Constructor)]
pub struct InContext<'a, 'b, 'w, 's, 'c, 'z> {
    pub context: &'a mut Context,
    pub commands: &'b mut Commands<'w, 's>,
    pub mod_: &'a Mod,
    pub w: &'c WorldConfig,
    pub world: &'z crate::world::World,
    pub ingame: &'z crate::ingame::state::State,
}

#[derive(Debug, Clone, EnumIter, Default)]
pub enum Tab {
    #[default]
    States,
    Cursor,
    Components,
    Leftclick,
    Weapons,
    Mixing,
}

impl Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::States => f.write_str("States"),
            Tab::Cursor => f.write_str("Cursor"),
            Tab::Components => f.write_str("Components"),
            Tab::Leftclick => f.write_str("Left click"),
            Tab::Weapons => f.write_str("Weapons"),
            Tab::Mixing => f.write_str("Mixing"),
        }
    }
}

impl<'a, 'b, 'w, 's, 'c, 'z> egui_dock::TabViewer for InContext<'a, 'b, 'w, 's, 'c, 'z> {
    type Tab = Tab;

    fn id(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::Id {
        egui_dock::egui::Id::new(tab.to_string())
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        tab.to_string().into()
    }

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, tab: &mut Self::Tab) {
        let context = &mut self.context;
        let commands = &mut self.commands;
        let mod_ = self.mod_;
        let w = self.w;
        let world = self.world;
        let ingame = self.ingame;

        match tab {
            Tab::States => context.ui_states(w, ui, commands, mod_, world, ingame),
            Tab::Cursor => context.ui_cursor(w, ui, commands, mod_, world, ingame),
            Tab::Components => context.ui_components(w, ui, commands, mod_, world, ingame),
            Tab::Leftclick => context.ui_left_click(w, ui, commands, mod_, world, ingame),
            Tab::Weapons => context.weapons(w, ui, commands, mod_, world, ingame),
            Tab::Mixing => context.mixing(ui, commands),
        }
    }
}

#[derive(Debug, Default)]
pub struct DebugBattleWindowPlugin;

impl Plugin for DebugBattleWindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugBattleWindow>()
            .add_observer(on_mounted_window)
            .add_observer(on_unmounted_window)
            .add_observer(refresh::on_refresh)
            .add_systems(Update, refresh::trigger_refresh);
    }
}

fn on_mounted_window(mounted: On<MountedWindow>, mut commands: Commands) {
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let crate::window::Window::BattleDebug(_) = &mounted.0 {
        commands.trigger(Refresh)
    }
}

fn on_unmounted_window(unmounted: On<UnmountedWindow>, mut window: ResMut<DebugBattleWindow>) {
    // Store unmounted debug window to reuse it later when want to display it again
    #[allow(irrefutable_let_patterns)] // TODO: no more irrefutable when more windows
    if let crate::window::Window::BattleDebug(window_) = &unmounted.0 {
        window.0 = Some((**window_).clone())
    }
}
