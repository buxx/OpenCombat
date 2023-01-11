use std::collections::HashMap;

use ggez::{
    graphics::{self, spritebatch::SpriteBatch, DrawParam, FilterMode, Image, MeshBuilder, Rect},
    Context, GameResult,
};
use keyframe::{AnimationSequence, EasingFunction};
use keyframe_derive::CanTween;

use crate::{
    entity::{soldier::Soldier, vehicle::Vehicle},
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
    soldiers_sprites_batch: SpriteBatch,
    vehicles_sprites_batch: SpriteBatch,
    explosions_sprites_batch: SpriteBatch,
    // Squad menu, etc
    ui_batch: SpriteBatch,
    // Map background sprite batch
    map_background_batch: SpriteBatch,
    // Map interiors sprite batch
    map_interiors_batch: SpriteBatch,
    // Map decor sprite batches
    map_decor_batches: Vec<SpriteBatch>,
    // Soldiers animations
    soldier_animation_sequences: HashMap<SoldierIndex, AnimationSequence<TweenableRect>>,
    // Explosion animations
    explosion_sequences: Vec<(WorldPoint, AnimationSequence<TweenableRect>)>,
}

impl Graphics {
    pub fn new(ctx: &mut Context, map: &Map) -> GameResult<Graphics> {
        let soldiers_sprites_batch = create_sprites_batch(SOLDIERS_FILE_PATH, ctx)?;
        let vehicles_sprites_batch = create_sprites_batch(VEHICLES_FILE_PATH, ctx)?;
        let explosions_sprites_batch = create_sprites_batch(EXPLOSIONS_FILE_PATH, ctx)?;
        let ui_batch = create_ui_batch(ctx)?;
        let map_background_batch = map::get_map_background_batch(ctx, map)?;
        let map_interiors_batch = map::get_map_interiors_batch(ctx, map)?;
        let map_decor_batches = map::get_map_decor_batch(ctx, map)?;

        Ok(Graphics {
            soldiers_sprites_batch,
            vehicles_sprites_batch,
            explosions_sprites_batch,
            ui_batch,
            map_background_batch,
            map_interiors_batch,
            map_decor_batches,
            soldier_animation_sequences: HashMap::new(),
            explosion_sequences: vec![],
        })
    }

    pub fn append_sprites_batch(&mut self, sprite: graphics::DrawParam) {
        self.soldiers_sprites_batch.add(sprite);
    }

    pub fn append_vehicles_batch(&mut self, sprite: graphics::DrawParam) {
        self.vehicles_sprites_batch.add(sprite);
    }

    pub fn append_explosions_batch(&mut self, sprite: graphics::DrawParam) {
        self.explosions_sprites_batch.add(sprite);
    }

    pub fn append_interior(&mut self, sprite: graphics::DrawParam) {
        self.map_interiors_batch.add(sprite);
    }

    pub fn append_ui_batch(&mut self, sprite: graphics::DrawParam) {
        self.ui_batch.add(sprite);
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

        // TODO depending of a lot of things like soldier type, physical behavior, etc
        // let relative_start_x = 0. / config::SCENE_ITEMS_SPRITE_SHEET_WIDTH;
        // let relative_start_y = 0. / config::SCENE_ITEMS_SPRITE_SHEET_HEIGHT;
        // let relative_tile_width = 12. / config::SCENE_ITEMS_SPRITE_SHEET_WIDTH;
        // let relative_tile_height = 12. / config::SCENE_ITEMS_SPRITE_SHEET_HEIGHT;
        vec![graphics::DrawParam::new()
            .src(current_frame_src)
            .rotation(soldier.get_looking_direction().0)
            .offset(Offset::new(0.5, 0.5).to_vec2())
            .dest(soldier.get_world_point().to_vec2())]
    }

    pub fn vehicle_sprites(
        &self,
        _vehicle_index: VehicleIndex,
        vehicle: &Vehicle,
    ) -> Vec<graphics::DrawParam> {
        let sprite_infos = VehicleGraphicInfos::from_type(vehicle.get_type());
        let shadow_offset = RelativeOffset::new(0.05, 0.05).to_vec2();
        let mut sprites = vec![];

        // Vehicle body shadow
        let body_sprite = DrawParam::new()
            .src(sprite_infos.chassis().shadow_version().to_rect())
            .offset(Offset::new(0.5, 0.5).to_vec2())
            .rotation(vehicle.get_chassis_orientation().0)
            .dest(vehicle.get_world_point().to_vec2() + shadow_offset);
        sprites.push(body_sprite);

        // Vehicle body
        let body_sprite = DrawParam::new()
            .src(sprite_infos.chassis().to_rect())
            .offset(Offset::new(0.5, 0.5).to_vec2())
            .rotation(vehicle.get_chassis_orientation().0)
            .dest(vehicle.get_world_point().to_vec2());
        sprites.push(body_sprite);

        // Main turret
        if let Some((turret_offset, turret_sprite_info)) = sprite_infos.main_turret() {
            let turret_shadow_sprite = DrawParam::new()
                .src(turret_sprite_info.shadow_version().to_rect())
                .dest(vehicle.get_world_point().to_vec2())
                .rotation(vehicle.get_chassis_orientation().0)
                .offset(Offset::new(0.5, 0.5).to_vec2() + turret_offset.to_vec2() + shadow_offset);
            sprites.push(turret_shadow_sprite);

            let turret_sprite = DrawParam::new()
                .src(turret_sprite_info.to_rect())
                .dest(vehicle.get_world_point().to_vec2())
                .rotation(vehicle.get_chassis_orientation().0)
                .offset(Offset::new(0.5, 0.5).to_vec2() + turret_offset.to_vec2());
            sprites.push(turret_sprite);
        }

        sprites
    }

    pub fn explosion_sprites(&self) -> Vec<graphics::DrawParam> {
        let mut sprites = vec![];

        for (world_point, explosion_sequence) in &self.explosion_sequences {
            let current_frame_src: Rect = explosion_sequence.now_strict().unwrap().into();
            sprites.push(
                graphics::DrawParam::new()
                    .src(current_frame_src)
                    .offset(Offset::new(0.5, 0.5).to_vec2())
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

    pub fn draw_scene(
        &mut self,
        ctx: &mut Context,
        draw_decor: bool,
        draw_param: graphics::DrawParam,
    ) -> GameResult {
        // Map background sprites
        graphics::draw(ctx, &self.map_background_batch, draw_param)?;

        // Map interior sprites
        graphics::draw(ctx, &self.map_interiors_batch, draw_param)?;

        // Entities, explosions, etc. sprites
        graphics::draw(ctx, &self.soldiers_sprites_batch, draw_param)?;
        graphics::draw(ctx, &self.vehicles_sprites_batch, draw_param)?;
        graphics::draw(ctx, &self.explosions_sprites_batch, draw_param)?;

        // Draw decor like Trees
        if draw_decor {
            for decor_batch in self.map_decor_batches.iter() {
                graphics::draw(ctx, decor_batch, draw_param)?;
            }
        }

        Ok(())
    }

    pub fn draw_ui(
        &mut self,
        ctx: &mut Context,
        draw_param: graphics::DrawParam,
        mesh_builder: MeshBuilder,
    ) -> GameResult {
        // Different meshes
        if let Ok(mesh) = mesh_builder.build(ctx) {
            graphics::draw(ctx, &mesh, draw_param)?;
        };

        // Squad menu, etc
        graphics::draw(ctx, &self.ui_batch, draw_param)?;

        Ok(())
    }

    pub fn clear(&mut self, ctx: &mut Context) {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.soldiers_sprites_batch.clear();
        self.vehicles_sprites_batch.clear();
        self.explosions_sprites_batch.clear();
        self.ui_batch.clear();
    }

    pub fn clear_map_interiors_batch(&mut self) {
        self.map_interiors_batch.clear();
    }

    pub fn tick(&mut self, ctx: &Context) {
        self.update(ctx);
    }

    pub fn react(&mut self, message: GraphicsMessage) {
        match message {
            GraphicsMessage::PushExplosionAnimation(point, type_) => {
                self.push_explosion_animation(point, type_)
            }
            GraphicsMessage::RemoveExplosionAnimation(point) => {
                self.remove_explosion_animation(point)
            }
        }
    }
}

pub fn create_sprites_batch(file_path: &str, ctx: &mut Context) -> GameResult<SpriteBatch> {
    let sprites_image = Image::new(ctx, file_path)?;
    let sprites_batch = SpriteBatch::new(sprites_image);

    Ok(sprites_batch)
}

pub fn create_ui_batch(ctx: &mut Context) -> GameResult<SpriteBatch> {
    let mut ui_image = Image::new(ctx, UI_FILE_PATH)?;
    ui_image.set_filter(FilterMode::Nearest); // because pixel art
    let ui_batch = SpriteBatch::new(ui_image);

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
    relative_start_x: f32,
    relative_start_y: f32,
    relative_tile_width: f32,
    relative_tile_height: f32,
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
            relative_start_x: start_x / sprite_sheet_width,
            relative_start_y: start_y / sprite_sheet_height,
            relative_tile_width: width / sprite_sheet_width,
            relative_tile_height: height / sprite_sheet_height,
        }
    }

    pub fn to_rect(&self) -> Rect {
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
            relative_start_x: self.relative_start_x + self.relative_tile_width,
            relative_start_y: self.relative_start_y,
            relative_tile_width: self.relative_tile_width,
            relative_tile_height: self.relative_tile_height,
        }
    }
}
