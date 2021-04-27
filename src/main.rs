use ggez;
use ggez::event::{KeyCode, MouseButton};
use ggez::graphics;
use ggez::graphics::{Color, DrawMode, FillOptions, MeshBuilder, StrokeOptions};
use ggez::timer::check_update_time;
use ggez::{event, input};
use ggez::{Context, GameResult};
use glam::Vec2;
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::path;

// TODO: create a ScenePosition and a WindowPosition to be more explicit
type Point2 = Vec2;
type Vector2 = Vec2;

const TARGET_FPS: u32 = 60; // execute update code 60x per seconds
const META_EACH: u32 = 20; // execute meta code each 20 frames
const PHYSICS_EACH: u32 = 10; // execute physics code each 10 frames
const ANIMATE_EACH: u32 = 60; // execute animate code each 30 frames
const SPRITE_EACH: u32 = 10; // change sprite animation tile 30 frames
const MAX_FRAME_I: u32 = 4294967295; // max of frame_i used to calculate ticks
const DISPLAY_OFFSET_BY: f32 = 3.0; // pixel offset by tick when player move screen display
const DISPLAY_OFFSET_BY_SPEED: f32 = 10.0; // pixel offset by tick when player move screen display with speed
const SCENE_ITEMS_SPRITE_SHEET_WIDTH: f32 = 800.0; // Width of sprite sheet
const SCENE_ITEMS_SPRITE_SHEET_HEIGHT: f32 = 600.0; // Height of sprite sheet
const UI_SPRITE_SHEET_WIDTH: f32 = 800.0; // Width of sprite sheet
const UI_SPRITE_SHEET_HEIGHT: f32 = 600.0; // Height of sprite sheet
const GRID_TILE_WIDTH: f32 = 5.0; // Width of one grid tile
const GRID_TILE_HEIGHT: f32 = 5.0; // Height of one grid tile
const DEFAULT_SELECTED_SQUARE_SIDE: f32 = 14.0;
const DEFAULT_SELECTED_SQUARE_SIDE_HALF: f32 = DEFAULT_SELECTED_SQUARE_SIDE / 2.0;
const SCENE_ITEMS_CHANGE_ERR_MSG: &str = "scene_items content change !";

#[derive(Eq, PartialEq, Hash)]
pub struct GridPosition {
    x: i32,
    y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

fn grid_position_from_position(position: &Point2) -> GridPosition {
    GridPosition::new(
        (position.x / GRID_TILE_WIDTH) as i32,
        (position.y / GRID_TILE_HEIGHT) as i32,
    )
}

enum UiItem {
    SceneItemMenu,
}

enum SceneItemMenuItem {
    Move,
}

struct UiSpriteInfo {
    relative_start_x: f32,
    relative_start_y: f32,
    relative_width: f32,
    relative_height: f32,
    width: f32,
    height: f32,
}

impl UiSpriteInfo {
    pub fn from_type(type_: UiItem) -> Self {
        match type_ {
            UiItem::SceneItemMenu => Self {
                relative_start_x: 0.0,
                relative_start_y: 0.0,
                relative_width: 71.0 / UI_SPRITE_SHEET_WIDTH,
                relative_height: 68.0 / UI_SPRITE_SHEET_HEIGHT,
                width: 71.0,
                height: 68.0,
            },
        }
    }

    pub fn as_draw_param(&self) -> graphics::DrawParam {
        graphics::DrawParam::new().src(graphics::Rect::new(
            self.relative_start_x,
            self.relative_start_y,
            self.relative_width,
            self.relative_height,
        ))
    }

    pub fn which_item_clicked(
        &self,
        menu_position: Point2,
        click_position: Point2,
        scene_item: &SceneItem,
    ) -> Option<SceneItemMenuItem> {
        Some(SceneItemMenuItem::Move)
    }
}

struct SpriteInfo {
    relative_start_y: f32,
    relative_tile_width: f32,
    relative_tile_height: f32,
    tile_count: u16,
    tile_width: f32,
    tile_height: f32,
    _half_tile_width: f32,
    _half_tile_height: f32,
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
            relative_start_y: start_y / SCENE_ITEMS_SPRITE_SHEET_HEIGHT,
            relative_tile_width: tile_width / SCENE_ITEMS_SPRITE_SHEET_WIDTH,
            relative_tile_height: tile_height / SCENE_ITEMS_SPRITE_SHEET_HEIGHT,
            tile_count,
            tile_width,
            tile_height,
            _half_tile_width: tile_width / 2.0,
            _half_tile_height: tile_height / 2.0,
        }
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

enum SceneItemType {
    Soldier,
}

impl ItemState {
    pub fn new(current_behavior: ItemBehavior) -> Self {
        Self { current_behavior }
    }
}

struct SceneItem {
    type_: SceneItemType,
    position: Point2,
    grid_position: GridPosition,
    state: ItemState,
    meta_events: Vec<MetaEvent>,
    current_frame: u16,
}

impl SceneItem {
    pub fn new(type_: SceneItemType, position: Point2, state: ItemState) -> Self {
        Self {
            type_,
            position: position.clone(),
            grid_position: grid_position_from_position(&position.clone()),
            state,
            meta_events: vec![],
            current_frame: 0,
        }
    }

    pub fn sprite_info(&self) -> SpriteInfo {
        SpriteInfo::from_type(&self.sprite_type())
    }

    pub fn tick_sprite(&mut self) {
        self.current_frame += 1;
        // TODO: good way to have sprite info ? performant ?
        if self.current_frame >= self.sprite_info().tile_count {
            self.current_frame = 0;
        }
    }

    pub fn as_draw_param(&self, current_frame: f32) -> graphics::DrawParam {
        let sprite_info = self.sprite_info();
        graphics::DrawParam::new()
            .src(graphics::Rect::new(
                current_frame as f32 * sprite_info.relative_tile_width,
                sprite_info.relative_start_y,
                sprite_info.relative_tile_width,
                sprite_info.relative_tile_height,
            ))
            .rotation(90.0f32.to_radians())
            .offset(Point2::new(0.5, 0.5))
    }

    pub fn sprite_type(&self) -> SpriteType {
        // Here some logical about state, nature (soldier, tank, ...) and current behavior to
        // determine sprite type
        match self.state.current_behavior {
            ItemBehavior::Crawling => SpriteType::CrawlingSoldier,
            ItemBehavior::Walking(_) => SpriteType::WalkingSoldier,
            ItemBehavior::Standing(_) => SpriteType::StandingSoldier,
        }
    }
}

#[derive(Debug)]
enum PhysicEvent {
    Explosion,
}

#[derive(Debug)]
enum MetaEvent {
    FearAboutExplosion,
}

#[derive(Debug)]
enum UserEvent {
    Click(Point2),                 // Window coordinates
    RightClick(Point2),            // Window coordinates
    AreaSelection(Point2, Point2), // Window coordinates
}

enum SceneItemPrepareOrder {
    Move(usize), // scene_item usize
}

struct MainState {
    // time
    frame_i: u32,

    // display
    display_offset: Point2,
    sprite_sheet_batch: graphics::spritebatch::SpriteBatch,
    map_batch: graphics::spritebatch::SpriteBatch,
    ui_batch: graphics::spritebatch::SpriteBatch,

    // scene items
    scene_items: Vec<SceneItem>,
    scene_items_by_grid_position: HashMap<GridPosition, Vec<usize>>,

    // events
    physics_events: Vec<PhysicEvent>,

    // user interactions
    left_click_down: Option<Point2>,
    right_click_down: Option<Point2>,
    current_cursor_position: Point2,
    user_events: Vec<UserEvent>,
    selected_scene_items: Vec<usize>,         // scene_item usize
    scene_item_menu: Option<(usize, Point2)>, // scene_item usize, display_at
    scene_item_prepare_order: Option<SceneItemPrepareOrder>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let sprite_sheet = graphics::Image::new(ctx, "/sprite_sheet.png").unwrap();
        let sprite_sheet_batch = graphics::spritebatch::SpriteBatch::new(sprite_sheet);
        let map = graphics::Image::new(ctx, "/map1bg.png").unwrap();
        let map_batch = graphics::spritebatch::SpriteBatch::new(map);
        let ui = graphics::Image::new(ctx, "/ui.png").unwrap();
        let ui_batch = graphics::spritebatch::SpriteBatch::new(ui);

        let mut scene_items = vec![];
        for x in 0..1 {
            for y in 0..4 {
                let current_behavior = if y % 2 == 0 {
                    ItemBehavior::Walking(vec_from_angle(90.0))
                } else {
                    ItemBehavior::Crawling
                };

                scene_items.push(SceneItem::new(
                    SceneItemType::Soldier,
                    Point2::new((x as f32 * 24.0) + 100.0, (y as f32 * 24.0) + 100.0),
                    ItemState::new(current_behavior),
                ));
            }
        }

        let mut main_state = MainState {
            frame_i: 0,
            display_offset: Point2::new(0.0, 0.0),
            sprite_sheet_batch,
            map_batch,
            ui_batch,
            scene_items,
            scene_items_by_grid_position: HashMap::new(),
            physics_events: vec![],
            left_click_down: None,
            right_click_down: None,
            current_cursor_position: Point2::new(0.0, 0.0),
            user_events: vec![],
            selected_scene_items: vec![],
            scene_item_menu: None,
            scene_item_prepare_order: None,
        };

        for (i, scene_item) in main_state.scene_items.iter().enumerate() {
            let grid_position = grid_position_from_position(&scene_item.position);
            main_state
                .scene_items_by_grid_position
                .entry(grid_position)
                .or_default()
                .push(i);
        }

        Ok(main_state)
    }

    fn inputs(&mut self, ctx: &Context) {
        let display_offset_by =
            if input::keyboard::is_mod_active(ctx, input::keyboard::KeyMods::SHIFT) {
                DISPLAY_OFFSET_BY_SPEED
            } else {
                DISPLAY_OFFSET_BY
            };

        if input::keyboard::is_key_pressed(ctx, KeyCode::Left) {
            self.display_offset.x += display_offset_by;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Right) {
            self.display_offset.x -= display_offset_by;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Up) {
            self.display_offset.y += display_offset_by;
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::Down) {
            self.display_offset.y -= display_offset_by;
        }

        while let Some(user_event) = self.user_events.pop() {
            match user_event {
                UserEvent::Click(click_position) => {
                    let scene_position = Point2::new(
                        click_position.x - self.display_offset.x,
                        click_position.y - self.display_offset.y,
                    );
                    self.selected_scene_items.drain(..);
                    if let Some(scene_item_usize) =
                        self.get_first_scene_item_for_position(&scene_position)
                    {
                        self.selected_scene_items.push(scene_item_usize);
                    }

                    if let Some(scene_item_prepare_order) = &self.scene_item_prepare_order {
                        // TODO: Add order to scene_item
                        self.scene_item_prepare_order = None;
                    }

                    // FIXME BS NOW: interpreter sur quel element du menu on a click ...
                    if let Some((scene_item_usize, menu_position)) = self.scene_item_menu {
                        let menu_sprite_info = UiSpriteInfo::from_type(UiItem::SceneItemMenu);
                        let scene_item = self
                            .scene_items
                            .get(scene_item_usize)
                            .expect(SCENE_ITEMS_CHANGE_ERR_MSG);
                        if click_position.x >= menu_position.x
                            && click_position.x <= menu_position.x + menu_sprite_info.width
                            && click_position.y >= menu_position.y
                            && click_position.y <= menu_position.y + menu_sprite_info.height
                        {
                            if let Some(menu_item) = menu_sprite_info.which_item_clicked(
                                menu_position,
                                click_position,
                                scene_item,
                            ) {
                                match menu_item {
                                    SceneItemMenuItem::Move => {
                                        self.scene_item_prepare_order =
                                            Some(SceneItemPrepareOrder::Move(scene_item_usize));
                                        self.scene_item_menu = None;
                                    }
                                }
                            }
                        } else {
                            self.scene_item_menu = None;
                        }
                    };
                }
                UserEvent::AreaSelection(from, to) => {
                    let scene_from = Point2::new(
                        from.x - self.display_offset.x,
                        from.y - self.display_offset.y,
                    );
                    let scene_to =
                        Point2::new(to.x - self.display_offset.x, to.y - self.display_offset.y);
                    self.selected_scene_items.drain(..);
                    self.selected_scene_items
                        .extend(self.get_scene_items_for_area(&scene_from, &scene_to));
                }
                UserEvent::RightClick(position) => {
                    if let Some(scene_item_usize) =
                        self.get_first_scene_item_for_position(&position)
                    {
                        if self.selected_scene_items.contains(&scene_item_usize) {
                            let scene_item = self
                                .scene_items
                                .get(scene_item_usize)
                                .expect(SCENE_ITEMS_CHANGE_ERR_MSG);
                            self.scene_item_menu =
                                Some((scene_item_usize, scene_item.position.clone()))
                        }
                    }
                }
            }
        }
    }

    // TODO: manage errors
    fn physics(&mut self) {
        // Scene items movements
        for scene_item in self.scene_items.iter_mut() {
            match scene_item.state.current_behavior {
                ItemBehavior::Walking(vector) => {
                    // TODO ici il faut calculer le déplacement réél (en fonction des ticks, etc ...)
                    scene_item.position.x += 1.0;
                    scene_item.grid_position = grid_position_from_position(&scene_item.position);
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

    fn position_with_display_offset(&self, position: &Point2) -> Point2 {
        Point2::new(
            position.x + self.display_offset.x,
            position.y + self.display_offset.y,
        )
    }

    fn get_first_scene_item_for_position(&self, position: &Point2) -> Option<usize> {
        // TODO: if found multiple: select nearest
        for (i, scene_item) in self.scene_items.iter().enumerate() {
            let sprite_info = scene_item.sprite_info();
            if scene_item.position.x >= position.x - sprite_info.tile_width
                && scene_item.position.x <= position.x + sprite_info.tile_width
                && scene_item.position.y >= position.y - sprite_info.tile_height
                && scene_item.position.y <= position.y + sprite_info.tile_height
            {
                return Some(i);
            }
        }

        None
    }

    fn get_scene_items_for_area(&self, from: &Point2, to: &Point2) -> Vec<usize> {
        let mut selection = vec![];

        for (i, scene_item) in self.scene_items.iter().enumerate() {
            if scene_item.position.x >= from.x
                && scene_item.position.x <= to.x
                && scene_item.position.y >= from.y
                && scene_item.position.y <= to.y
            {
                selection.push(i);
            }
        }

        selection
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, TARGET_FPS) {
            self.inputs(ctx);

            // TODO: meta: calculer par ex qui voit qui (soldat voit un ennemi: ajouter l'event a vu
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

        let mut scene_mesh_builder = MeshBuilder::new();

        for scene_item in self.scene_items.iter() {
            self.sprite_sheet_batch.add(
                scene_item
                    .as_draw_param(scene_item.current_frame as f32)
                    .dest(scene_item.position.clone()),
            );
            scene_mesh_builder.circle(
                DrawMode::fill(),
                scene_item.position.clone(),
                2.0,
                2.0,
                graphics::WHITE,
            )?;
        }

        for i in &self.selected_scene_items {
            let selected_scene_item = self.scene_items.get(*i).expect(SCENE_ITEMS_CHANGE_ERR_MSG);
            scene_mesh_builder.rectangle(
                DrawMode::Stroke(StrokeOptions::default()),
                graphics::Rect::new(
                    selected_scene_item.position.x - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                    selected_scene_item.position.y - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                    DEFAULT_SELECTED_SQUARE_SIDE,
                    DEFAULT_SELECTED_SQUARE_SIDE,
                ),
                graphics::GREEN,
            )?;
        }

        if let Some(left_click_down) = self.left_click_down {
            if left_click_down != self.current_cursor_position {
                scene_mesh_builder.rectangle(
                    DrawMode::fill(),
                    graphics::Rect::new(
                        left_click_down.x - self.display_offset.x,
                        left_click_down.y - self.display_offset.y,
                        self.current_cursor_position.x - left_click_down.x,
                        self.current_cursor_position.y - left_click_down.y,
                    ),
                    graphics::GREEN,
                )?;
            }

            scene_mesh_builder.circle(
                DrawMode::fill(),
                left_click_down,
                2.0,
                2.0,
                graphics::YELLOW,
            )?;
        }

        if let Some((_, position)) = self.scene_item_menu {
            self.ui_batch.add(
                UiSpriteInfo::from_type(UiItem::SceneItemMenu)
                    .as_draw_param()
                    .dest(position),
            );
        }

        if let Some(scene_item_prepare_order) = &self.scene_item_prepare_order {
            match scene_item_prepare_order {
                SceneItemPrepareOrder::Move(scene_item_usize) => {
                    let scene_item = self
                        .scene_items
                        .get(*scene_item_usize)
                        .expect(SCENE_ITEMS_CHANGE_ERR_MSG);
                    scene_mesh_builder.line(
                        &vec![scene_item.position.clone(), self.current_cursor_position],
                        2.0,
                        graphics::WHITE,
                    )?;
                }
            }
        }

        self.map_batch.add(
            graphics::DrawParam::new()
                .src(graphics::Rect::new(0.0, 0.0, 1.0, 1.0))
                .dest(Point2::new(0.0, 0.0)),
        );

        let scene_mesh = scene_mesh_builder.build(ctx)?;
        graphics::draw(
            ctx,
            &self.map_batch,
            graphics::DrawParam::new()
                .dest(self.position_with_display_offset(&Point2::new(0.0, 0.0))),
        )?;
        graphics::draw(
            ctx,
            &self.sprite_sheet_batch,
            graphics::DrawParam::new()
                .dest(self.position_with_display_offset(&Point2::new(0.0, 0.0))),
        )?;
        graphics::draw(
            ctx,
            &scene_mesh,
            graphics::DrawParam::new()
                .dest(self.position_with_display_offset(&Point2::new(0.0, 0.0))),
        )?;
        graphics::draw(
            ctx,
            &self.ui_batch,
            graphics::DrawParam::new()
                .dest(self.position_with_display_offset(&Point2::new(0.0, 0.0))),
        )?;

        self.sprite_sheet_batch.clear();
        self.map_batch.clear();
        self.ui_batch.clear();
        graphics::present(ctx)?;

        println!("FPS: {}", ggez::timer::fps(ctx));
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => {
                self.left_click_down = Some(Point2::new(x, y));
            }
            MouseButton::Right => {
                self.right_click_down = Some(Point2::new(x, y));
            }
            MouseButton::Middle => {}
            MouseButton::Other(_) => {}
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => {
                if let Some(left_click_down) = self.left_click_down {
                    if left_click_down == Point2::new(x, y) {
                        self.user_events.push(UserEvent::Click(left_click_down));
                    } else {
                        let from = Point2::new(
                            cmp::min(left_click_down.x as i32, x as i32) as f32,
                            cmp::min(left_click_down.y as i32, y as i32) as f32,
                        );
                        let to = Point2::new(
                            cmp::max(left_click_down.x as i32, x as i32) as f32,
                            cmp::max(left_click_down.y as i32, y as i32) as f32,
                        );
                        self.user_events.push(UserEvent::AreaSelection(from, to));
                    }
                }
                self.left_click_down = None;
            }
            MouseButton::Right => {
                if let Some(right_click_down) = self.right_click_down {
                    self.user_events
                        .push(UserEvent::RightClick(right_click_down));
                }
            }
            MouseButton::Middle => {}
            MouseButton::Other(_) => {}
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.current_cursor_position = Point2::new(x, y);
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
    let (mut ctx, event_loop) = cb.build()?;

    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
