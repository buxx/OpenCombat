use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::timer::check_update_time;
use ggez::{Context, GameResult};
use glam::Vec2;
use std::env;
use std::path;

type Vector2 = Vec2;

const TARGET_FPS: u32 = 60; // execute update code 60x per seconds
const META_EACH: u32 = 20; // execute meta code each 20 frames
const PHYSICS_EACH: u32 = 10; // execute physics code each 10 frames
const ANIMATE_EACH: u32 = 60; // execute animate code each 30 frames
const SPRITE_EACH: u32 = 10; // change sprite animation tile 30 frames
const MAX_FRAME_I: u32 = 4294967295; // max of frame_i used to calculate ticks

const SPRITE_SHEET_WIDTH: f32 = 800.0;
const SPRITE_SHEET_HEIGHT: f32 = 600.0;

fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

struct SpriteInfo {
    relative_start_y: f32,
    relative_tile_width: f32,
    relative_tile_height: f32,
    tile_count: u16,
    tile_width: f32,
    tile_height: f32,
    half_tile_width: f32,
    half_tile_height: f32,
}

impl SpriteInfo {
    // TODO: ask on rust community if this is performant, or how to make it static
    pub fn from_type(type_: &SpriteType) -> Self {
        let (start_y, tile_width, tile_height, tile_count) = match type_ {
            SpriteType::WalkingSoldier => (12.0, 12.0, 12.0, 8),
            SpriteType::CrawlingSoldier => (26.0, 26.0, 26.0, 8),
            SpriteType::StandingSoldier => (0.0, 12.0, 12.0, 1),
        };

        Self {
            relative_start_y: start_y / SPRITE_SHEET_HEIGHT,
            relative_tile_width: tile_width / SPRITE_SHEET_WIDTH,
            relative_tile_height: tile_height / SPRITE_SHEET_HEIGHT,
            tile_count,
            tile_width,
            tile_height,
            half_tile_width: tile_width / 2.0,
            half_tile_height: tile_height / 2.0,
        }
    }

    pub fn as_draw_param(&self, current_frame: f32) -> graphics::DrawParam {
        graphics::DrawParam::new().src(graphics::Rect::new(
            current_frame as f32 * self.relative_tile_width,
            self.relative_start_y,
            self.relative_tile_width,
            self.relative_tile_height,
        ))
    }
}

enum SpriteType {
    WalkingSoldier,
    CrawlingSoldier,
    StandingSoldier,
}

enum ItemBehavior {
    Standing(u32), // since
    Crawling,
    Walking(Vector2),
}

struct ItemState {
    current_behavior: ItemBehavior,
}

impl ItemState {
    pub fn new(current_behavior: ItemBehavior) -> Self {
        Self { current_behavior }
    }

    pub fn sprite_type(&self) -> SpriteType {
        // Here some logical about state and current behavior to determine sprite type
        match self.current_behavior {
            ItemBehavior::Crawling => SpriteType::CrawlingSoldier,
            ItemBehavior::Walking(_) => SpriteType::WalkingSoldier,
            ItemBehavior::Standing(_) => SpriteType::StandingSoldier,
        }
    }
}

struct SceneItem {
    position: na::Point2<f32>,
    state: ItemState,
    meta_events: Vec<MetaEvent>,
    current_frame: u16,
}

impl SceneItem {
    pub fn new(position: na::Point2<f32>, state: ItemState) -> Self {
        let sprite_type = state.sprite_type();
        Self {
            position,
            state,
            meta_events: vec![],
            current_frame: 0,
        }
    }

    pub fn sprite_info(&self) -> SpriteInfo {
        SpriteInfo::from_type(&self.state.sprite_type())
    }

    pub fn tick_sprite(&mut self) {
        self.current_frame += 1;
        // TODO: good way to have sprite info ? performant ?
        if self.current_frame >= self.sprite_info().tile_count {
            self.current_frame = 0;
        }
    }

    pub fn position_with_tile_decal(&self) -> na::Point2<f32> {
        let sprite_info = self.sprite_info();
        na::Point2::new(
            self.position.x - sprite_info.half_tile_width,
            self.position.y - sprite_info.half_tile_height,
        )
    }
}

enum PhysicEvent {
    Explosion,
}

enum MetaEvent {
    FearAboutExplosion,
}

struct MainState {
    frame_i: u32,
    scene_items_sprite_batch: graphics::spritebatch::SpriteBatch,
    scene_items: Vec<SceneItem>,
    physics_events: Vec<PhysicEvent>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image = graphics::Image::new(ctx, "/sprite_sheet.png").unwrap();
        let batch = graphics::spritebatch::SpriteBatch::new(image);

        let mut scene_items = vec![];
        for x in 0..1 {
            for y in 0..4 {
                let current_behavior = if y % 2 == 0 {
                    ItemBehavior::Walking(vec_from_angle(90.0))
                } else {
                    ItemBehavior::Crawling
                };

                scene_items.push(SceneItem::new(
                    na::Point2::new(x as f32 * 24.0, y as f32 * 24.0),
                    ItemState::new(current_behavior),
                ));
            }
        }

        let s = MainState {
            frame_i: 0,
            scene_items_sprite_batch: batch,
            scene_items,
            physics_events: vec![],
        };
        Ok(s)
    }

    // TODO: manage errors
    fn physics(&mut self) {
        // Scene items movements
        for scene_item in self.scene_items.iter_mut() {
            match scene_item.state.current_behavior {
                ItemBehavior::Walking(vector) => {
                    // TODO ici il faut calculer le déplacement réél (en fonction des ticks, etc ...)
                    scene_item.position.x += 1.0;
                }
                _ => {}
            }
        }

        // (FAKE) Drop a bomb to motivate stop move
        if self.frame_i % 600 == 0 && self.frame_i != 0 {
            self.physics_events.push(PhysicEvent::Explosion);
        }
    }

    fn metas(&mut self) {
        for physic_event in &self.physics_events {
            match physic_event {
                PhysicEvent::Explosion => {
                    for scene_item in self.scene_items.iter_mut() {
                        scene_item.meta_events.push(MetaEvent::FearAboutExplosion);
                    }
                }
            }
        }
    }

    fn animate(&mut self) {
        // TODO: ici il faut reflechir a comment organiser les comportements

        for scene_item in self.scene_items.iter_mut() {
            for meta_event in &scene_item.meta_events {
                match meta_event {
                    MetaEvent::FearAboutExplosion => {
                        scene_item.state = ItemState::new(ItemBehavior::Standing(self.frame_i));
                    }
                }
            }

            match scene_item.state.current_behavior {
                ItemBehavior::Crawling => {
                    scene_item.state = ItemState::new(ItemBehavior::Walking(vec_from_angle(90.0)));
                }
                ItemBehavior::Walking(_) => {
                    scene_item.state = ItemState::new(ItemBehavior::Crawling);
                }
                ItemBehavior::Standing(since) => {
                    if self.frame_i - since >= 120 {
                        scene_item.state =
                            ItemState::new(ItemBehavior::Walking(vec_from_angle(90.0)));
                    }
                }
            }

            scene_item.meta_events.drain(..);
        }
    }

    fn tick_sprites(&mut self) {
        for scene_item in self.scene_items.iter_mut() {
            scene_item.tick_sprite();
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, TARGET_FPS) {
            // FIXME: gérer ici la maj des physics, animate, meta etc
            // meta: calculer par ex qui voit qui (soldat voit un ennemi: ajouter l'event a vu
            // ennemi, dans animate il se mettra a tirer)
            let tick_sprite = self.frame_i % SPRITE_EACH == 0;
            let tick_animate = self.frame_i % ANIMATE_EACH == 0;
            let tick_physics = self.frame_i % PHYSICS_EACH == 0;
            let tick_meta = self.frame_i % META_EACH == 0;

            // Apply moves, explosions, etc
            if tick_physics {
                self.physics();
            }

            // Generate meta events according to physics events and current physic state
            if tick_meta {
                self.metas();
            }

            // Animate scene items according to meta events
            if tick_animate {
                self.animate();
            };

            // Change scene items tiles
            if tick_sprite {
                self.tick_sprites();
            }

            // Increment frame counter
            self.frame_i += 1;
            if self.frame_i >= MAX_FRAME_I {
                self.frame_i = 0;
            }

            // Empty physics event
            self.physics_events.drain(..);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        for scene_item in self.scene_items.iter() {
            self.scene_items_sprite_batch.add(
                scene_item
                    .sprite_info()
                    .as_draw_param(scene_item.current_frame as f32)
                    .dest(scene_item.position_with_tile_decal()),
            );
        }
        graphics::draw(
            ctx,
            &self.scene_items_sprite_batch,
            graphics::DrawParam::new().dest(na::Point2::new(0.0, 0.0)),
        )?;

        self.scene_items_sprite_batch.clear();
        graphics::present(ctx)?;

        println!("FPS: {}", ggez::timer::fps(ctx));
        Ok(())
    }
}

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("oc", "bux")
        .add_resource_path(resource_dir)
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0));
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
