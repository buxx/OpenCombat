use ggez;
use ggez::event;
use ggez::graphics;
use ggez::nalgebra as na;
use ggez::timer::check_update_time;
use ggez::{Context, GameResult};
use std::env;
use std::path;

struct SpriteInfo {
    source: graphics::Rect,
    tile_count: u16,
    relative_tile_width: f32,
    relative_tile_height: f32,
}

impl SpriteInfo {
    pub fn new(source: graphics::Rect, tile_count: u16) -> Self {
        let relative_tile_width: f32 = 1.0 / tile_count as f32;
        Self {
            source,
            tile_count,
            relative_tile_width,
            relative_tile_height: 1.0,
        }
    }

    pub fn from_type(type_: &SpriteType) -> Self {
        match type_ {
            SpriteType::WalkingSoldier => Self::new(graphics::Rect::new(0.0, 0.0, 128.0, 24.0), 7),
        }
    }
}

enum SpriteType {
    WalkingSoldier,
}

fn sprite_batch_part_from_sprite_info(
    sprite_info: &SpriteInfo,
    frame_i: u32,
) -> graphics::DrawParam {
    let src = graphics::Rect::new(
        frame_i as f32 * sprite_info.relative_tile_width,
        0.0,
        sprite_info.relative_tile_width,
        sprite_info.relative_tile_height,
    );
    graphics::DrawParam::new().src(src)
}

struct SceneItem {
    current_sprite_type: SpriteType,
    position: na::Point2<f32>,
}

impl SceneItem {
    pub fn new(current_sprite_type: SpriteType, position: na::Point2<f32>) -> Self {
        Self {
            current_sprite_type,
            position,
        }
    }
}

struct MainState {
    scene_items_sprite_batch: graphics::spritebatch::SpriteBatch,
    scene_items: Vec<SceneItem>,
    i: u32,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let image = graphics::Image::new(ctx, "/test.png").unwrap();
        let batch = graphics::spritebatch::SpriteBatch::new(image);

        let mut scene_items = vec![];
        for x in 0..10 {
            for y in 0..10 {
                scene_items.push(SceneItem::new(
                    SpriteType::WalkingSoldier,
                    na::Point2::new(x as f32 * 24.0, y as f32 * 24.0),
                ));
            }
        }

        let s = MainState {
            scene_items_sprite_batch: batch,
            scene_items,
            i: 0,
        };
        Ok(s)
    }
}
impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if check_update_time(ctx, 5) {
            self.i += 1;
        }
        if self.i > 6 {
            self.i = 0;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);

        for scene_item in self.scene_items.iter() {
            let sprite_info = SpriteInfo::from_type(&scene_item.current_sprite_type);
            let sprite_batch_part = sprite_batch_part_from_sprite_info(&sprite_info, self.i).dest(scene_item.position.clone());
            self.scene_items_sprite_batch.add(sprite_batch_part);
        }
        graphics::draw(
            ctx,
            &self.scene_items_sprite_batch,
            graphics::DrawParam::new().dest(na::Point2::new(0.0, 0.0)),
        );

        self.scene_items_sprite_batch.clear();
        graphics::present(ctx);

        println!("FPS: {}", ggez::timer::fps(ctx));
        Ok(())
    }
}
// TODO: spite i par objet, fabrication des sprite_info qu'une fois; channel pour modifs des objets ds update
pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ggez::ContextBuilder::new("oc", "bux").add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
