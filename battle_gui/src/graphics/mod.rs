use std::{collections::HashMap, fs};

use battle_core::{
    config::ServerConfig,
    entity::{soldier::Soldier, vehicle::Vehicle},
    graphics::vehicle::VehicleGraphicInfos,
    map::Map,
    types::{Scale, SoldierIndex, SquadUuid, VehicleIndex, WindowPoint, WorldPoint},
};
use ggez::{
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

use crate::{debug::DebugTerrain, ui::menu::squad_menu_sprite_info, RESOURCE_PATH};

use self::message::GraphicsMessage;

pub mod animation;
pub mod map;
pub mod message;
pub mod order;
pub mod soldier;

pub enum AssetsType {
    Soldiers,
    Vehicles,
    Explosions,
    Ui,
}

impl AssetsType {
    pub fn prefix(&self) -> &str {
        match self {
            AssetsType::Soldiers => "/soldiers",
            AssetsType::Vehicles => "/vehicles",
            AssetsType::Explosions => "/explosions",
            AssetsType::Ui => "/ui",
        }
    }
}

pub struct Graphics {
    // Sprites batches
    soldiers_batch: InstanceArray,
    soldiers_files: Vec<String>,
    soldiers_file: String,
    vehicles_batch: InstanceArray,
    vehicles_files: Vec<String>,
    vehicles_file: String,
    explosions_batch: InstanceArray,
    explosions_files: Vec<String>,
    explosions_file: String,
    // Squad menu, etc
    ui_batch: InstanceArray,
    ui_files: Vec<String>,
    ui_file: String,
    // Map background sprite batch
    map_background_batch: InstanceArray,
    // Map interiors sprite batch
    map_interiors_batch: InstanceArray,
    // Map decor sprite batches
    map_decor_batches: Vec<InstanceArray>,
    // Soldiers animations
    soldier_animation_sequences: HashMap<SoldierIndex, AnimationSequence<TweenableRect>>,
    // Explosion animations
    explosion_sequences: Vec<(WorldPoint, AnimationSequence<TweenableRect>)>,
    //
    debug_terrain_batch: InstanceArray,
    //
    debug_terrain_opacity_mesh_builder: MeshBuilder,
}

impl Graphics {
    pub fn new(ctx: &mut Context, map: &Map, config: &ServerConfig) -> GameResult<Graphics> {
        let soldiers_file = AssetsType::Soldiers.prefix().to_string() + ".png";
        let soldiers_files = collect_resources_by_prefix(AssetsType::Soldiers.prefix())?;
        let soldiers_batch = create_batch(&soldiers_file, ctx)?;

        let vehicles_file = AssetsType::Vehicles.prefix().to_string() + ".png";
        let vehicles_files = collect_resources_by_prefix(AssetsType::Vehicles.prefix())?;
        let vehicles_batch = create_batch(&vehicles_file, ctx)?;

        let explosions_file = AssetsType::Explosions.prefix().to_string() + ".png";
        let explosions_files = collect_resources_by_prefix(AssetsType::Explosions.prefix())?;
        let explosions_batch = create_batch(&explosions_file, ctx)?;

        let ui_file = AssetsType::Ui.prefix().to_string() + ".png";
        let ui_files = collect_resources_by_prefix(AssetsType::Ui.prefix())?;
        let ui_batch = create_batch(&ui_file, ctx)?;

        let map_background_batch = map::get_map_background_batch(ctx, map)?;
        let map_interiors_batch = map::get_map_interiors_batch(ctx, map)?;
        let map_decor_batches = map::get_map_decor_batch(ctx, map)?;
        let debug_terrain_batch = map::create_debug_terrain_batch(ctx, map)?;
        let debug_terrain_opacity_mesh_builder =
            map::create_debug_terrain_opacity_mesh_builder(map, config)?;

        Ok(Graphics {
            soldiers_batch,
            soldiers_files,
            soldiers_file,
            vehicles_batch,
            vehicles_files,
            vehicles_file,
            explosions_batch,
            explosions_files,
            explosions_file,
            ui_batch,
            ui_files,
            ui_file,
            map_background_batch,
            map_interiors_batch,
            map_decor_batches,
            soldier_animation_sequences: HashMap::new(),
            explosion_sequences: vec![],
            debug_terrain_batch,
            debug_terrain_opacity_mesh_builder,
        })
    }

    pub fn append_soldier_batch(&mut self, sprite: graphics::DrawParam) {
        self.soldiers_batch.push(sprite);
    }

    pub fn append_vehicles_batch(&mut self, sprite: graphics::DrawParam) {
        self.vehicles_batch.push(sprite);
    }

    pub fn append_explosions_batch(&mut self, sprite: graphics::DrawParam) {
        self.explosions_batch.push(sprite);
    }

    pub fn append_interior(&mut self, sprite: graphics::DrawParam) {
        self.map_interiors_batch.push(sprite);
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
    ) -> Vec<graphics::DrawParam> {
        let current_frame_src: Rect = self
            .soldier_animation_sequences
            .get(&soldier.uuid())
            .expect("Shared state must be consistent")
            .now_strict()
            .unwrap()
            .into();

        const SOLDIER_SPRITE_OFFSET: (f32, f32) = (
            SOLDIER_TILE_WIDTH as f32 * 0.5,
            SOLDIER_TILE_HEIGHT as f32 * 0.5,
        );

        vec![graphics::DrawParam::new()
            .src(current_frame_src)
            .rotation(soldier.get_looking_direction().0)
            .offset(Vec2::from(SOLDIER_SPRITE_OFFSET))
            .dest(
                draw_to
                    .and_then(|p| Some(p.to_vec2()))
                    .unwrap_or(soldier.get_world_point().to_vec2()),
            )]
    }

    pub fn vehicle_sprites(
        &self,
        _vehicle_index: VehicleIndex,
        vehicle: &Vehicle,
    ) -> Vec<graphics::DrawParam> {
        let vehicle_sprite_infos = VehicleGraphicInfos::from_type(vehicle.get_type());
        let mut sprites = vec![];

        const VEHICLE_SPRITE_OFFSET: (f32, f32) =
            (VEHICLE_TILE_WIDTH * 0.5, VEHICLE_TILE_HEIGHT * 0.5);
        const VEHICLE_SPRITE_SHADOW_OFFSET: (f32, f32) =
            (VEHICLE_TILE_WIDTH * 0.05, VEHICLE_TILE_HEIGHT * 0.05);

        // Vehicle body shadow
        let body_shadow_sprite = vehicle_sprite_infos.chassis().shadow_version();
        let body_shadow_draw = DrawParam::new()
            .offset(Vec2::from(VEHICLE_SPRITE_OFFSET))
            .dest(vehicle.get_world_point().to_vec2() + Vec2::from(VEHICLE_SPRITE_SHADOW_OFFSET))
            .src(Rect::from(body_shadow_sprite.relative_rect().to_array()))
            .rotation(vehicle.get_chassis_orientation().0);
        sprites.push(body_shadow_draw);

        // Vehicle body
        let body_sprite = vehicle_sprite_infos.chassis();
        let body_draw = DrawParam::new()
            .offset(Vec2::from(VEHICLE_SPRITE_OFFSET))
            .src(Rect::from(body_sprite.relative_rect().to_array()))
            .rotation(vehicle.get_chassis_orientation().0)
            .dest(vehicle.get_world_point().to_vec2());
        sprites.push(body_draw);

        // Main turret
        if let Some((turret_offset, turret_sprite_info)) = vehicle_sprite_infos.main_turret() {
            let turret_shadow_sprite = turret_sprite_info.shadow_version();
            let turret_shadow_draw = DrawParam::new()
                .offset(
                    Vec2::from(VEHICLE_SPRITE_OFFSET)
                        + turret_sprite_info.abs_offset(&turret_offset).to_vec2()
                        + Vec2::from(VEHICLE_SPRITE_SHADOW_OFFSET),
                )
                .src(Rect::from(turret_shadow_sprite.relative_rect().to_array()))
                .dest(vehicle.get_world_point().to_vec2())
                .rotation(vehicle.get_chassis_orientation().0);
            sprites.push(turret_shadow_draw);

            let turret_sprite = turret_sprite_info;
            let turret_draw = DrawParam::new()
                .offset(
                    Vec2::from(VEHICLE_SPRITE_OFFSET)
                        + turret_sprite_info.abs_offset(&turret_offset).to_vec2(),
                )
                .src(Rect::from(turret_sprite.relative_rect().to_array()))
                .dest(vehicle.get_world_point().to_vec2())
                .rotation(vehicle.get_chassis_orientation().0);
            sprites.push(turret_draw);
        }

        sprites
    }

    pub fn explosion_sprites(&self) -> Vec<graphics::DrawParam> {
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
                    .dest(world_point.to_vec2()),
            )
        }

        sprites
    }

    pub fn squad_menu_sprites(
        &self,
        to_point: WindowPoint,
        cursor_point: WindowPoint,
        _squad_id: SquadUuid,
    ) -> Vec<graphics::DrawParam> {
        squad_menu_sprite_info().as_draw_params(&to_point, &cursor_point)
    }

    pub fn draw_map(&mut self, canvas: &mut Canvas, draw_param: graphics::DrawParam) -> GameResult {
        // Map background sprites
        if self.map_background_batch.instances().len() > 0 {
            canvas.draw(&self.map_background_batch, draw_param);
        }

        // Map interior sprites
        if self.map_interiors_batch.instances().len() > 0 {
            canvas.draw(&self.map_interiors_batch, draw_param);
        }

        Ok(())
    }

    pub fn draw_decor(
        &mut self,
        canvas: &mut Canvas,
        draw_decor: bool,
        draw_param: graphics::DrawParam,
    ) -> GameResult {
        // Draw decor like Trees
        if draw_decor {
            for decor_batch in self.map_decor_batches.iter() {
                canvas.draw(decor_batch, draw_param);
            }
        }

        Ok(())
    }

    pub fn draw_units(
        &mut self,
        canvas: &mut Canvas,
        draw_param: graphics::DrawParam,
    ) -> GameResult {
        // Entities, explosions, etc. sprites
        if self.soldiers_batch.instances().len() > 0 {
            canvas.draw(&self.soldiers_batch, draw_param);
        }
        if self.vehicles_batch.instances().len() > 0 {
            canvas.draw(&self.vehicles_batch, draw_param);
        }
        if self.explosions_batch.instances().len() > 0 {
            canvas.draw(&self.explosions_batch, draw_param);
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
        if self.ui_batch.instances().len() > 0 {
            canvas.draw(&self.ui_batch, draw_param);
        }

        Ok(())
    }

    pub fn clear(&mut self) {
        self.soldiers_batch.clear();
        self.vehicles_batch.clear();
        self.explosions_batch.clear();
        self.ui_batch.clear();
    }

    pub fn clear_map_interiors_batch(&mut self) {
        self.map_interiors_batch.clear();
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
            GraphicsMessage::RemoveExplosionAnimation(point) => {
                self.remove_explosion_animation(point)
            }
            GraphicsMessage::RecomputeDebugTerrainOpacity => {
                self.debug_terrain_opacity_mesh_builder =
                    map::create_debug_terrain_opacity_mesh_builder(map, config)?;
            }
            GraphicsMessage::ReloadSoldiersAsset => {
                self.soldiers_batch = create_batch(&self.soldiers_file, ctx)?;
            }
            GraphicsMessage::ReloadVehiclesAsset => {
                self.vehicles_batch = create_batch(&self.vehicles_file, ctx)?;
            }
            GraphicsMessage::ReloadExplosionsAsset => {
                self.explosions_batch = create_batch(&self.explosions_file, ctx)?;
            }
            GraphicsMessage::ReloadUiAsset => {
                self.ui_batch = create_batch(&self.ui_file, ctx)?;
            }
            GraphicsMessage::ReloadAll => {
                self.soldiers_files = collect_resources_by_prefix(AssetsType::Soldiers.prefix())?;
                self.soldiers_batch = create_batch(&self.soldiers_file, ctx)?;

                self.vehicles_files = collect_resources_by_prefix(AssetsType::Vehicles.prefix())?;
                self.vehicles_batch = create_batch(&self.vehicles_file, ctx)?;

                self.explosions_files =
                    collect_resources_by_prefix(AssetsType::Explosions.prefix())?;
                self.explosions_batch = create_batch(&self.explosions_file, ctx)?;

                self.ui_files = collect_resources_by_prefix(AssetsType::Ui.prefix())?;
                self.ui_batch = create_batch(&self.ui_file, ctx)?;

                self.map_background_batch = map::get_map_background_batch(ctx, map)?;
                self.map_interiors_batch = map::get_map_interiors_batch(ctx, map)?;
                self.map_decor_batches = map::get_map_decor_batch(ctx, map)?;
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
}

pub fn create_batch(file_path: &str, ctx: &mut Context) -> GameResult<InstanceArray> {
    let image = Image::from_path(ctx, file_path)?;
    let batch = InstanceArray::new(ctx, image);
    Ok(batch)
}

pub fn collect_resources_by_prefix(prefix: &str) -> GameResult<Vec<String>> {
    let mut resources = vec![];
    let prefix = match prefix.strip_prefix("/") {
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
                if file_name.starts_with(prefix) && file_name.ends_with(".png") {
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
