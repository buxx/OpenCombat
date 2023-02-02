use std::collections::HashMap;

use ggez::{
    graphics::{self, Canvas, DrawParam, Image, InstanceArray, Mesh, MeshBuilder, Rect},
    Context, GameResult,
};
use glam::Vec2;
use keyframe::{AnimationSequence, EasingFunction};
use keyframe_derive::CanTween;

use crate::{
    config::Config,
    debug::DebugTerrain,
    entity::{soldier::Soldier, vehicle::Vehicle},
    graphics::explosion::TILE_HEIGHT as EXPLOSION_TILE_WIDTH,
    graphics::explosion::TILE_WIDTH as EXPLOSION_TILE_HEIGHT,
    graphics::soldier::TILE_HEIGHT as SOLDIER_TILE_HEIGHT,
    graphics::soldier::TILE_WIDTH as SOLDIER_TILE_WIDTH,
    graphics::vehicle::TILE_HEIGHT as VEHICLE_TILE_HEIGHT,
    graphics::vehicle::TILE_WIDTH as VEHICLE_TILE_WIDTH,
    map::Map,
    message::GraphicsMessage,
    types::*,
    ui::menu::squad_menu_sprite_info,
};

use self::vehicle::VehicleGraphicInfos;

pub mod animation;
pub mod explosion;
mod map;
pub mod soldier;
pub mod vehicle;

const SOLDIERS_FILE_PATH: &'static str = "/soldiers.png";
const VEHICLES_FILE_PATH: &'static str = "/vehicles.png";
const EXPLOSIONS_FILE_PATH: &'static str = "/explosions.png";
const UI_FILE_PATH: &'static str = "/ui.png";

pub struct Graphics {
    // Sprites batches
    soldiers_batch: InstanceArray,
    vehicles_batch: InstanceArray,
    explosions_batch: InstanceArray,
    // Squad menu, etc
    ui_batch: InstanceArray,
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
    pub fn new(ctx: &mut Context, map: &Map, config: &Config) -> GameResult<Graphics> {
        let soldiers_batch = create_batch(SOLDIERS_FILE_PATH, ctx)?;
        let vehicles_batch = create_batch(VEHICLES_FILE_PATH, ctx)?;
        let explosions_batch = create_batch(EXPLOSIONS_FILE_PATH, ctx)?;
        let ui_batch = create_ui_batch(ctx)?;
        let map_background_batch = map::get_map_background_batch(ctx, map)?;
        let map_interiors_batch = map::get_map_interiors_batch(ctx, map)?;
        let map_decor_batches = map::get_map_decor_batch(ctx, map)?;
        let debug_terrain_batch = map::create_debug_terrain_batch(ctx, map)?;
        let debug_terrain_opacity_mesh_builder =
            map::create_debug_terrain_opacity_mesh_builder(map, config)?;

        Ok(Graphics {
            soldiers_batch,
            vehicles_batch,
            explosions_batch,
            ui_batch,
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

    pub fn soldier_sprites(
        &self,
        soldier_index: SoldierIndex,
        soldier: &Soldier,
    ) -> Vec<graphics::DrawParam> {
        let current_frame_src: Rect = self
            .soldier_animation_sequences
            .get(&soldier_index)
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
            .dest(soldier.get_world_point().to_vec2())]
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
            .src(body_shadow_sprite.relative_rect())
            .rotation(vehicle.get_chassis_orientation().0);
        sprites.push(body_shadow_draw);

        // Vehicle body
        let body_sprite = vehicle_sprite_infos.chassis();
        let body_draw = DrawParam::new()
            .offset(Vec2::from(VEHICLE_SPRITE_OFFSET))
            .src(body_sprite.relative_rect())
            .rotation(vehicle.get_chassis_orientation().0)
            .dest(vehicle.get_world_point().to_vec2());
        sprites.push(body_draw);

        // Main turret
        if let Some((turret_offset, turret_sprite_info)) = vehicle_sprite_infos.main_turret() {
            let turret_shadow_sprite = turret_sprite_info.shadow_version();
            let turret_shadow_draw = DrawParam::new()
                .offset(
                    Vec2::from(VEHICLE_SPRITE_OFFSET)
                        + turret_offset
                            .to_absolute_from_sprite(&turret_sprite_info)
                            .to_vec2()
                        + Vec2::from(VEHICLE_SPRITE_SHADOW_OFFSET),
                )
                .src(turret_shadow_sprite.relative_rect())
                .dest(vehicle.get_world_point().to_vec2())
                .rotation(vehicle.get_chassis_orientation().0);
            sprites.push(turret_shadow_draw);

            let turret_sprite = turret_sprite_info;
            let turret_draw = DrawParam::new()
                .offset(
                    Vec2::from(VEHICLE_SPRITE_OFFSET)
                        + turret_offset
                            .to_absolute_from_sprite(&turret_sprite_info)
                            .to_vec2(),
                )
                .src(turret_sprite.relative_rect())
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
        canvas.draw(&self.map_background_batch, draw_param);

        // Map interior sprites
        canvas.draw(&self.map_interiors_batch, draw_param);

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
        canvas.draw(&self.soldiers_batch, draw_param);
        canvas.draw(&self.vehicles_batch, draw_param);
        canvas.draw(&self.explosions_batch, draw_param);

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
        canvas.draw(&self.ui_batch, draw_param);

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
        self.update(ctx);
    }

    pub fn react(
        &mut self,
        message: GraphicsMessage,
        map: &Map,
        config: &Config,
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

pub fn create_ui_batch(ctx: &mut Context) -> GameResult<InstanceArray> {
    let image = Image::from_path(ctx, UI_FILE_PATH)?;
    let ui_batch = InstanceArray::new(ctx, image);
    Ok(ui_batch)
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

pub struct SpriteInfo {
    pub start_x: f32,
    pub start_y: f32,
    pub tile_width: f32,
    pub tile_height: f32,
    pub relative_start_x: f32,
    pub relative_start_y: f32,
    pub relative_tile_width: f32,
    pub relative_tile_height: f32,
}

impl SpriteInfo {
    pub fn new(
        start_x: f32,
        start_y: f32,
        width: f32,
        height: f32,
        sprite_sheet_width: f32,
        sprite_sheet_height: f32,
    ) -> Self {
        Self {
            start_x,
            start_y,
            tile_width: width,
            tile_height: height,
            relative_start_x: start_x / sprite_sheet_width,
            relative_start_y: start_y / sprite_sheet_height,
            relative_tile_width: width / sprite_sheet_width,
            relative_tile_height: height / sprite_sheet_height,
        }
    }

    pub fn relative_rect(&self) -> Rect {
        Rect::new(
            self.relative_start_x,
            self.relative_start_y,
            self.relative_tile_width,
            self.relative_tile_height,
        )
    }

    pub fn shadow_version(&self) -> Self {
        // Convention is shadow sprite is at right of regular sprite
        Self {
            start_x: self.start_x + self.tile_width,
            start_y: self.start_y + self.tile_height,
            tile_width: self.tile_width,
            tile_height: self.tile_height,
            relative_start_x: self.relative_start_x + self.relative_tile_width,
            relative_start_y: self.relative_start_y,
            relative_tile_width: self.relative_tile_width,
            relative_tile_height: self.relative_tile_height,
        }
    }
}
