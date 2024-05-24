pub mod utils;
use std::{collections::HashMap, fs, path::PathBuf};

use battle_core::{
    config::ServerConfig,
    entity::{soldier::Soldier, vehicle::Vehicle},
    game::{control::MapControl, weapon::WeaponSprite},
    graphics::{
        cannon_blast::{
            TILE_HEIGHT as CANNON_BLAST_TILE_HEIGHT, TILE_WIDTH as CANNON_BLAST_TILE_WIDTH,
        },
        vehicle::VehicleGraphicInfos,
    },
    map::Map,
    types::{Angle, Scale, SoldierIndex, SquadUuid, VehicleIndex, WindowPoint, WorldPoint},
};
use ggez::{
    conf::{FullscreenType, WindowMode},
    graphics::{self, Canvas, DrawParam, Image, InstanceArray, Mesh, MeshBuilder, Rect},
    Context, GameError, GameResult,
};
use glam::Vec2;
use keyframe::{AnimationSequence, EasingFunction};
use keyframe_derive::CanTween;

use battle_core::{
    graphics::explosion::TILE_HEIGHT as EXPLOSION_TILE_WIDTH,
    graphics::explosion::TILE_WIDTH as EXPLOSION_TILE_HEIGHT,
    graphics::soldier::TILE_HEIGHT as SOLDIER_TILE_HEIGHT,
    graphics::soldier::TILE_WIDTH as SOLDIER_TILE_WIDTH,
    graphics::vehicle::TILE_HEIGHT as VEHICLE_TILE_HEIGHT,
    graphics::vehicle::TILE_WIDTH as VEHICLE_TILE_WIDTH,
};
use oc_core::resources::RESOURCE_PATH;

use crate::{
    debug::DebugTerrain,
    ui::{component::Component, hud::Hud, menu::squad_menu_sprite_info},
};

use self::{
    background::{Background, BackgroundBuilder},
    batch::QualifiedBatch,
    cannon_blasts::{CannonBlasts, CannonBlastsBuilder},
    decors::{DecorDrawRule::*, Decors, DecorsBuilder},
    explosions::{Explosions, ExplosionsBuilder},
    interiors::{Interiors, InteriorsBuilder},
    message::GraphicsMessage,
    minimap::MinimapBuilder,
    qualified::Zoom,
    soldier::{SoldierAnimationSequence, Soldiers, SoldiersBuilder},
    vehicles::{Vehicles, VehiclesBuilder},
    weapons::Weapons,
};

pub mod animation;
pub mod background;
pub mod batch;
pub mod cannon_blasts;
pub mod decors;
pub mod explosions;
pub mod flag;
pub mod interiors;
pub mod map;
pub mod message;
pub mod minimap;
pub mod order;
pub mod qualified;
pub mod soldier;
pub mod squad;
pub mod vehicles;
pub mod weapons;

pub enum AssetsType {
    Soldiers,
    Weapon(WeaponSprite),
    CannonBlasts,
    Vehicles,
    Explosions,
    Ui,
    Flags,
}

impl AssetsType {
    pub fn prefix(&self) -> &str {
        match self {
            AssetsType::Soldiers => "/soldiers",
            AssetsType::Weapon(type_) => type_.prefix(),
            AssetsType::CannonBlasts => "/cannon_blasts",
            AssetsType::Vehicles => "/vehicles",
            AssetsType::Explosions => "/explosions",
            AssetsType::Ui => "/ui",
            AssetsType::Flags => "/flags",
        }
    }

    pub fn path(&self) -> PathBuf {
        PathBuf::from(format!("{}.png", self.prefix()))
    }
}

pub struct Graphics {
    // Sprites batches
    soldiers: Soldiers,
    soldiers_files: Vec<String>,
    soldiers_file: String,
    weapons: Weapons,
    vehicles: Vehicles,
    vehicles_files: Vec<String>,
    vehicles_file: String,
    explosions: Explosions,
    explosions_files: Vec<String>,
    explosions_file: String,
    cannon_blasts: CannonBlasts,
    // Squad menu, etc
    ui_batch: InstanceArray,
    ui_files: Vec<String>,
    ui_file: String,
    // Map background sprite batch
    background: Background,
    dark_background: Background,
    dark_background_first: bool,
    minimap: InstanceArray,
    flags: InstanceArray,
    // Map interiors sprite batch
    interiors: Interiors,
    // Map decor sprite batches
    decor: Decors,
    dark_decor: Decors,
    // Soldiers animations
    soldier_animation_sequences: HashMap<SoldierIndex, SoldierAnimationSequence>,
    // Explosion animations
    explosion_sequences: Vec<(WorldPoint, AnimationSequence<TweenableRect>)>,
    // Cannon blasts animations
    canon_blast_sequences: Vec<(WorldPoint, Angle, AnimationSequence<TweenableRect>)>,
    //
    debug_terrain_batch: InstanceArray,
    //
    debug_terrain_opacity_mesh_builder: MeshBuilder,
}

impl Graphics {
    pub fn new(
        ctx: &mut Context,
        map: &Map,
        config: &ServerConfig,
        a_control: &MapControl,
        b_control: &MapControl,
    ) -> GameResult<Graphics> {
        let soldiers_file = AssetsType::Soldiers.prefix().to_string() + ".png";
        let soldiers_files = collect_resources_by_prefix(AssetsType::Soldiers.prefix())?;
        let soldiers = SoldiersBuilder::new(ctx).build()?;

        let weapons = Weapons::new(ctx)?;

        let vehicles_file = AssetsType::Vehicles.prefix().to_string() + ".png";
        let vehicles_files = collect_resources_by_prefix(AssetsType::Vehicles.prefix())?;
        let vehicles = VehiclesBuilder::new(ctx).build()?;

        let explosions_file = AssetsType::Explosions.prefix().to_string() + ".png";
        let explosions_files = collect_resources_by_prefix(AssetsType::Explosions.prefix())?;
        let explosions = ExplosionsBuilder::new(ctx).build()?;

        let cannon_blasts = CannonBlastsBuilder::new(ctx).build()?;

        let ui_file = AssetsType::Ui.prefix().to_string() + ".png";
        let flags_file = AssetsType::Flags.prefix().to_string() + ".png";
        let ui_files = collect_resources_by_prefix(AssetsType::Ui.prefix())?;
        let ui_batch = create_batch(&ui_file, ctx)?;
        let flags = create_batch(&flags_file, ctx)?;

        let background = BackgroundBuilder::new(ctx, map).build()?;
        let dark_background = BackgroundBuilder::new(ctx, map).dark(true).build()?;
        let map_interiors_batch = InteriorsBuilder::new(ctx, map).build()?;
        let minimap = MinimapBuilder::new(ctx, map).build()?;
        let decor = DecorsBuilder::new(ctx, map)
            .rule(DrawOnly(a_control.clone()))
            .build()?;
        let dark_decor = DecorsBuilder::new(ctx, map)
            .dark(true)
            .rule(DrawOnly(b_control.clone()))
            .build()?;
        let debug_terrain_batch = map::create_debug_terrain_batch(ctx, map)?;
        let debug_terrain_opacity_mesh_builder =
            map::create_debug_terrain_opacity_mesh_builder(map, config)?;

        Ok(Graphics {
            soldiers,
            soldiers_files,
            soldiers_file,
            weapons,
            vehicles,
            vehicles_files,
            vehicles_file,
            explosions,
            explosions_files,
            explosions_file,
            cannon_blasts,
            ui_batch,
            ui_files,
            ui_file,
            background,
            dark_background,
            dark_background_first: false,
            minimap,
            flags,
            interiors: map_interiors_batch,
            decor,
            dark_decor,
            soldier_animation_sequences: HashMap::new(),
            explosion_sequences: vec![],
            canon_blast_sequences: vec![],
            debug_terrain_batch,
            debug_terrain_opacity_mesh_builder,
        })
    }

    pub fn append_ui_batch(&mut self, sprite: graphics::DrawParam) {
        self.ui_batch.push(sprite);
    }

    pub fn extend_ui_batch(&mut self, sprites: Vec<graphics::DrawParam>) {
        for sprite in sprites {
            self.append_ui_batch(sprite)
        }
    }

    pub fn soldiers_file_mut(&mut self) -> &mut String {
        &mut self.soldiers_file
    }

    pub fn soldiers_files(&self) -> &Vec<String> {
        &self.soldiers_files
    }

    pub fn vehicles_file_mut(&mut self) -> &mut String {
        &mut self.vehicles_file
    }

    pub fn vehicles_files(&self) -> &Vec<String> {
        &self.vehicles_files
    }

    pub fn explosions_file_mut(&mut self) -> &mut String {
        &mut self.explosions_file
    }

    pub fn explosions_files(&self) -> &Vec<String> {
        &self.explosions_files
    }

    pub fn ui_file_mut(&mut self) -> &mut String {
        &mut self.ui_file
    }

    pub fn ui_files(&self) -> &Vec<String> {
        &self.ui_files
    }

    pub fn soldier_sprites(
        &self,
        soldier: &Soldier,
        draw_to: Option<&WorldPoint>,
        zoom: &Zoom,
    ) -> (Vec<graphics::DrawParam>, Vec<graphics::DrawParam>) {
        let soldier_animation_sequence: &SoldierAnimationSequence = self
            .soldier_animation_sequences
            .get(&soldier.uuid())
            .expect("Shared state must be consistent");

        let soldier_current_frame_src: Rect = soldier_animation_sequence
            .soldier()
            .now_strict()
            .unwrap()
            .into();

        let soldier_sprite_offset: (f32, f32) = (
            SOLDIER_TILE_WIDTH as f32 * zoom.factor() * 0.5,
            SOLDIER_TILE_HEIGHT as f32 * zoom.factor() * 0.5,
        );

        let soldier_sprites = vec![graphics::DrawParam::new()
            .src(soldier_current_frame_src)
            .rotation(soldier.get_looking_direction().0)
            .offset(Vec2::from(soldier_sprite_offset))
            .dest(
                draw_to
                    .map(|p| p.to_vec2())
                    .unwrap_or(soldier.world_point().to_vec2())
                    * zoom.factor(),
            )];
        let weapon_sprites = if let Some(weapon_sequence) = soldier_animation_sequence.weapon() {
            let weapon_current_frame_src: Rect = weapon_sequence.now_strict().unwrap().into();
            vec![graphics::DrawParam::new()
                .src(weapon_current_frame_src)
                .rotation(soldier.get_looking_direction().0)
                .offset(Vec2::from(soldier_sprite_offset))
                .dest(
                    draw_to
                        .map(|p| p.to_vec2())
                        .unwrap_or(soldier.world_point().to_vec2())
                        * zoom.factor(),
                )]
        } else {
            vec![]
        };

        (soldier_sprites, weapon_sprites)
    }

    pub fn vehicle_sprites(
        &self,
        _vehicle_index: VehicleIndex,
        vehicle: &Vehicle,
        zoom: &Zoom,
    ) -> Vec<graphics::DrawParam> {
        let vehicle_sprite_infos = VehicleGraphicInfos::from_type(vehicle.type_());
        let mut sprites = vec![];

        let vehicle_sprite_offset: (f32, f32) = (
            VEHICLE_TILE_WIDTH * zoom.factor() * 0.5,
            VEHICLE_TILE_HEIGHT * zoom.factor() * 0.5,
        );
        let vehicle_sprite_shadow_offset: (f32, f32) = (
            VEHICLE_TILE_WIDTH * zoom.factor() * 0.05,
            VEHICLE_TILE_HEIGHT * zoom.factor() * 0.05,
        );

        // Vehicle body shadow
        let body_shadow_sprite = vehicle_sprite_infos.chassis().shadow_version();
        let body_shadow_draw = DrawParam::new()
            .offset(Vec2::from(vehicle_sprite_offset))
            .dest(
                vehicle.world_point().to_vec2() * zoom.factor()
                    + Vec2::from(vehicle_sprite_shadow_offset),
            )
            .src(Rect::from(body_shadow_sprite.relative_rect().to_array()))
            .rotation(vehicle.chassis_orientation().0);
        sprites.push(body_shadow_draw);

        // Vehicle body
        let body_sprite = vehicle_sprite_infos.chassis();
        let body_draw = DrawParam::new()
            .offset(Vec2::from(vehicle_sprite_offset))
            .src(Rect::from(body_sprite.relative_rect().to_array()))
            .rotation(vehicle.chassis_orientation().0)
            .dest(vehicle.world_point().to_vec2() * zoom.factor());
        sprites.push(body_draw);

        // Main turret
        if let Some((turret_offset, turret_sprite_info)) = vehicle_sprite_infos.main_turret() {
            let turret_shadow_sprite = turret_sprite_info.shadow_version();
            let turret_shadow_draw = DrawParam::new()
                .offset(
                    Vec2::from(vehicle_sprite_offset)
                        + turret_sprite_info.abs_offset(turret_offset).to_vec2()
                        + Vec2::from(vehicle_sprite_shadow_offset),
                )
                .src(Rect::from(turret_shadow_sprite.relative_rect().to_array()))
                .dest(vehicle.world_point().to_vec2() * zoom.factor())
                .rotation(vehicle.chassis_orientation().0);
            sprites.push(turret_shadow_draw);

            let turret_sprite = turret_sprite_info;
            let turret_draw = DrawParam::new()
                .offset(
                    Vec2::from(vehicle_sprite_offset)
                        + turret_sprite_info.abs_offset(turret_offset).to_vec2(),
                )
                .src(Rect::from(turret_sprite.relative_rect().to_array()))
                .dest(vehicle.world_point().to_vec2() * zoom.factor())
                .rotation(vehicle.chassis_orientation().0);
            sprites.push(turret_draw);
        }

        sprites
    }

    pub fn explosion_sprites(&self, zoom: &Zoom) -> Vec<graphics::DrawParam> {
        let mut sprites = vec![];

        const EXPLOSION_SPRITE_OFFSET: (f32, f32) = (
            EXPLOSION_TILE_WIDTH as f32 * 0.5,
            EXPLOSION_TILE_HEIGHT as f32 * 0.5,
        );

        for (world_point, explosion_sequence) in &self.explosion_sequences {
            let current_frame_src: Rect = explosion_sequence.now_strict().unwrap().into();
            sprites.push(
                graphics::DrawParam::new()
                    .src(current_frame_src)
                    .offset(Vec2::from(EXPLOSION_SPRITE_OFFSET))
                    .scale(Scale::new(0.25, 0.25).to_vec2())
                    .dest(world_point.to_vec2() * zoom.factor()),
            )
        }

        sprites
    }

    pub fn cannon_blasts_sprites(&self, zoom: &Zoom) -> Vec<graphics::DrawParam> {
        let mut sprites = vec![];
        for (world_point, angle, cannon_blast_sequence) in &self.canon_blast_sequences {
            let current_frame_src: Rect = cannon_blast_sequence.now_strict().unwrap().into();
            let cannon_blast_sprite_offset: (f32, f32) = (
                CANNON_BLAST_TILE_WIDTH as f32 * zoom.factor() * 0.5,
                CANNON_BLAST_TILE_HEIGHT as f32 * zoom.factor() * 0.5,
            );
            sprites.push(
                graphics::DrawParam::new()
                    .src(current_frame_src)
                    .rotation(angle.0)
                    .offset(Vec2::from(cannon_blast_sprite_offset))
                    .dest(world_point.to_vec2() * zoom.factor()),
            )
        }

        sprites
    }

    pub fn squad_menu_sprites(
        &self,
        to_point: WindowPoint,
        cursor_point: WindowPoint,
        _squad_ids: &[SquadUuid],
    ) -> Vec<graphics::DrawParam> {
        squad_menu_sprite_info().as_draw_params(&to_point, &cursor_point)
    }

    pub fn draw_map(
        &mut self,
        canvas: &mut Canvas,
        draw_param: graphics::DrawParam,
        zoom: &Zoom,
    ) -> GameResult {
        if self.dark_background_first {
            if !self.dark_background.drawable(zoom).instances().is_empty() {
                canvas.draw(self.dark_background.drawable(zoom), draw_param);
            }

            if !self.background.drawable(zoom).instances().is_empty() {
                canvas.draw(self.background.drawable(zoom), draw_param);
            }
        } else {
            if !self.background.drawable(zoom).instances().is_empty() {
                canvas.draw(self.background.drawable(zoom), draw_param);
            }

            if !self.dark_background.drawable(zoom).instances().is_empty() {
                canvas.draw(self.dark_background.drawable(zoom), draw_param);
            }
        }

        // Map interior sprites
        if !self.interiors.drawable(zoom).instances().is_empty() {
            canvas.draw(self.interiors.drawable(zoom), draw_param);
        }

        Ok(())
    }

    pub fn draw_flags(
        &mut self,
        canvas: &mut Canvas,
        draw_param: graphics::DrawParam,
    ) -> GameResult {
        if !self.flags.instances().is_empty() {
            canvas.draw(&self.flags, draw_param);
        }

        Ok(())
    }

    pub fn draw_decor(
        &mut self,
        canvas: &mut Canvas,
        draw_decor: bool,
        draw_param: graphics::DrawParam,
        zoom: &Zoom,
    ) -> GameResult {
        if draw_decor {
            self.decor
                .drawable(zoom)
                .iter()
                .for_each(|d| canvas.draw(d, draw_param));
            // TODO : Don't draw dark decor for now because two bugs : positions and black square ...
            // self.dark_decor
            //     .drawable(zoom)
            //     .iter()
            //     .for_each(|d| canvas.draw(d, draw_param));
        }

        Ok(())
    }

    pub fn draw_units(
        &mut self,
        canvas: &mut Canvas,
        draw_param: graphics::DrawParam,
        zoom: &Zoom,
    ) -> GameResult {
        // TODO : check is_empty not enough required since ggez fix
        // Entities, explosions, etc. sprites
        if !self.soldiers.drawable(zoom).instances().is_empty() {
            canvas.draw(self.soldiers.drawable(zoom), draw_param);
        }
        self.weapons.draw(canvas, zoom, draw_param);
        if !self.vehicles.drawable(zoom).instances().is_empty() {
            canvas.draw(self.vehicles.drawable(zoom), draw_param);
        }
        if !self.cannon_blasts.drawable(zoom).instances().is_empty() {
            canvas.draw(self.cannon_blasts.drawable(zoom), draw_param);
        }
        if !self.explosions.drawable(zoom).instances().is_empty() {
            canvas.draw(self.explosions.drawable(zoom), draw_param);
        }

        Ok(())
    }

    pub fn draw_ui(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        draw_param: graphics::DrawParam,
        mesh_builder: MeshBuilder,
    ) -> GameResult {
        // Different meshes
        canvas.draw(&Mesh::from_data(ctx, mesh_builder.build()), draw_param);

        // Squad menu, etc
        if !self.ui_batch.instances().is_empty() {
            canvas.draw(&self.ui_batch, draw_param);
        }

        Ok(())
    }

    pub fn set_map_dark_background_first(&mut self, map_dark_background_first: bool) {
        self.dark_background_first = map_dark_background_first;
    }

    pub fn clear(&mut self, zoom: &Zoom) {
        self.soldiers.clear(zoom);
        self.weapons.clear(zoom);
        self.vehicles.clear(zoom);
        self.explosions.clear(zoom);
        self.cannon_blasts.clear(zoom);
        self.background.clear(zoom);
        self.dark_background.clear(zoom);
        self.minimap.clear();
        self.minimap.push(DrawParam::new());
        self.ui_batch.clear();
        self.flags.clear();
    }

    pub fn clear_map_interiors_batch(&mut self, zoom: &Zoom) {
        self.interiors.clear(zoom);
    }

    pub fn tick(&mut self, ctx: &Context) {
        puffin::profile_scope!("tick_graphics");
        self.update(ctx);
    }

    pub fn react(
        &mut self,
        message: GraphicsMessage,
        map: &Map,
        config: &ServerConfig,
        ctx: &mut Context,
    ) -> GameResult<()> {
        match message {
            GraphicsMessage::PushExplosionAnimation(point, type_) => {
                self.push_explosion_animation(point, type_)
            }
            GraphicsMessage::PushCanonBlastAnimation(
                point,
                angle,
                type_,
                soldier_animation_type,
            ) => self.push_canon_blast_animation(point, angle, type_, soldier_animation_type),
            GraphicsMessage::RemoveExplosionAnimation(point) => {
                self.remove_explosion_animation(point)
            }
            GraphicsMessage::RemoveCanonBlastAnimation(point) => {
                self.remove_canon_blast_animation(point)
            }
            GraphicsMessage::RecomputeDebugTerrainOpacity => {
                self.debug_terrain_opacity_mesh_builder =
                    map::create_debug_terrain_opacity_mesh_builder(map, config)?;
            }
            GraphicsMessage::ReloadSoldiersAsset => {
                self.soldiers = SoldiersBuilder::new(ctx).build()?;
                // FIXME same for weapons, cannon_blasts
            }
            GraphicsMessage::ReloadVehiclesAsset => {
                self.vehicles = VehiclesBuilder::new(ctx).build()?;
            }
            GraphicsMessage::ReloadExplosionsAsset => {
                self.explosions = ExplosionsBuilder::new(ctx).build()?;
            }
            GraphicsMessage::ReloadUiAsset => {
                self.ui_batch = create_batch(&self.ui_file, ctx)?;
            }
            GraphicsMessage::ReloadAll => {
                self.soldiers_files = collect_resources_by_prefix(AssetsType::Soldiers.prefix())?;
                self.soldiers = SoldiersBuilder::new(ctx).build()?;
                // FIXME same for weapons, cannon_blasts

                self.vehicles_files = collect_resources_by_prefix(AssetsType::Vehicles.prefix())?;
                self.vehicles = VehiclesBuilder::new(ctx).build()?;

                self.explosions_files =
                    collect_resources_by_prefix(AssetsType::Explosions.prefix())?;
                self.explosions = ExplosionsBuilder::new(ctx).build()?;

                self.ui_files = collect_resources_by_prefix(AssetsType::Ui.prefix())?;
                self.ui_batch = create_batch(&self.ui_file, ctx)?;

                self.background = BackgroundBuilder::new(ctx, map).build()?;
                self.interiors = InteriorsBuilder::new(ctx, map).build()?;
                self.decor = DecorsBuilder::new(ctx, map).build()?;
                self.dark_decor = DecorsBuilder::new(ctx, map).dark(true).build()?;
                self.debug_terrain_batch = map::create_debug_terrain_batch(ctx, map)?;
                self.debug_terrain_opacity_mesh_builder =
                    map::create_debug_terrain_opacity_mesh_builder(map, config)?;
            }
        }

        GameResult::Ok(())
    }

    pub fn draw_debug_terrain(
        &mut self,
        ctx: &mut Context,
        canvas: &mut Canvas,
        debug: &DebugTerrain,
        draw_param: graphics::DrawParam,
    ) -> GameResult<()> {
        match debug {
            DebugTerrain::Tiles => {
                canvas.draw(&self.debug_terrain_batch, draw_param);
            }
            DebugTerrain::Opacity => {
                canvas.draw(
                    &Mesh::from_data(ctx, self.debug_terrain_opacity_mesh_builder.build()),
                    draw_param,
                );
            }
            DebugTerrain::None => {}
        };

        GameResult::Ok(())
    }

    pub fn background_mut(&mut self) -> &mut Background {
        &mut self.background
    }

    pub fn dark_background_mut(&mut self) -> &mut Background {
        &mut self.dark_background
    }

    pub fn interiors_mut(&mut self) -> &mut Interiors {
        &mut self.interiors
    }

    pub fn soldiers_mut(&mut self) -> &mut Soldiers {
        &mut self.soldiers
    }

    pub fn weapons_mut(&mut self) -> &mut Weapons {
        &mut self.weapons
    }

    pub fn vehicles_mut(&mut self) -> &mut Vehicles {
        &mut self.vehicles
    }

    pub fn explosions_mut(&mut self) -> &mut Explosions {
        &mut self.explosions
    }

    pub fn cannon_blasts_mut(&mut self) -> &mut CannonBlasts {
        &mut self.cannon_blasts
    }

    pub fn flags_mut(&mut self) -> &mut InstanceArray {
        &mut self.flags
    }

    pub fn battle_started(&mut self, ctx: &mut Context, map: &Map) -> GameResult<()> {
        // When battle started, all decors are displayed normally
        self.decor = DecorsBuilder::new(ctx, map).rule(DrawAll).build()?;
        Ok(())
    }

    pub fn draw_minimap(&self, ctx: &mut Context, canvas: &mut Canvas, hud: &Hud) -> GameResult {
        let dest = WindowPoint::new(hud.minimap().point(ctx).x, hud.minimap().point(ctx).y);
        canvas.draw(&self.minimap, DrawParam::new().dest(dest.to_vec2()));
        Ok(())
    }
}

pub fn fullscreen_mode() -> WindowMode {
    WindowMode::default().fullscreen_type(FullscreenType::Desktop)
}

pub fn windowed_mode() -> WindowMode {
    WindowMode::default()
        .dimensions(1024., 768.)
        .maximized(false)
        .resizable(true)
}

pub fn create_batch(file_path: &str, ctx: &mut Context) -> GameResult<InstanceArray> {
    let image = Image::from_path(ctx, file_path)?;
    let batch = InstanceArray::new(ctx, image);
    Ok(batch)
}

pub fn collect_resources_by_prefix(prefix: &str) -> GameResult<Vec<String>> {
    let mut resources = vec![];
    let prefix = match prefix.strip_prefix('/') {
        Some(prefix) => prefix,
        None => {
            return GameResult::Err(GameError::ResourceLoadError(format!(
                "Given prefix must start with '/' but is not : {}",
                prefix
            )))
        }
    };

    for path in match fs::read_dir(RESOURCE_PATH) {
        Ok(paths) => paths,
        Err(error) => {
            return GameResult::Err(GameError::ResourceLoadError(format!(
                "Error when trying to read resources folder : {}",
                error
            )))
        }
    } {
        match path {
            Ok(path) => {
                let file_name = path.file_name();
                let file_name = match file_name.to_str() {
                    Some(file_name) => file_name,
                    None => {
                        return GameResult::Err(GameError::ResourceLoadError(format!(
                            "Error when trying to read resource : {:?}",
                            file_name
                        )))
                    }
                };
                if file_name.starts_with(prefix)
                    && file_name.ends_with(".png")
                    && !file_name.ends_with("__HD.png")
                {
                    resources.push("/".to_string() + file_name)
                }
            }
            Err(error) => {
                return GameResult::Err(GameError::ResourceLoadError(format!(
                    "Error when trying to read path of resources folder : {}",
                    error
                )))
            }
        }
    }

    GameResult::Ok(resources)
}

#[derive(CanTween, Clone, Copy, Debug)]
/// necessary because we can't implement CanTween for graphics::Rect directly, as it's a foreign type
pub struct TweenableRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl TweenableRect {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        TweenableRect { x, y, w, h }
    }
}

impl From<TweenableRect> for Rect {
    fn from(t_rect: TweenableRect) -> Self {
        Rect {
            x: t_rect.x,
            y: t_rect.y,
            w: t_rect.w,
            h: t_rect.h,
        }
    }
}

/// A fancy easing function, tweening something into one of `frames` many discrete states.
/// The `pre_easing` is applied first, thereby making other `EasingFunction`s usable in the realm of frame-by-frame animation
struct AnimationFloor {
    pre_easing: Box<dyn EasingFunction + Send + Sync>,
    frames: i32,
}
impl EasingFunction for AnimationFloor {
    #[inline]
    fn y(&self, x: f64) -> f64 {
        (self.pre_easing.y(x) * (self.frames) as f64).floor() / (self.frames - 1) as f64
    }
}
