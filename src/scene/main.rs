use std::cmp;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use ggez::event::MouseButton;
use ggez::graphics::{Color, DrawMode, MeshBuilder, StrokeOptions, WHITE};
use ggez::input::keyboard::KeyCode;
use ggez::timer::check_update_time;
use ggez::{event, graphics, input, Context, GameResult};

use crate::behavior::animate::{digest_current_behavior, digest_current_order, digest_next_order};
use crate::behavior::order::Order;
use crate::behavior::ItemBehavior;
use crate::config::{
    ANIMATE_EACH, DEFAULT_SELECTED_SQUARE_SIDE, DEFAULT_SELECTED_SQUARE_SIDE_HALF,
    DISPLAY_OFFSET_BY, DISPLAY_OFFSET_BY_SPEED, INTERIORS_EACH, MAX_FRAME_I, META_EACH,
    PHYSICS_EACH, SCENE_ITEMS_CHANGE_ERR_MSG, SEEK_EACH, SPRITE_EACH, TARGET_FPS,
};
use crate::map::util::extract_image_from_tileset;
use crate::map::Map;
use crate::physics::item::produce_physics_messages_for_scene_item;
use crate::physics::path::find_path;
use crate::physics::util::{grid_point_from_scene_point, scene_point_from_window_point};
use crate::physics::util::{scene_point_from_grid_point, window_point_from_scene_point};
use crate::physics::GridPoint;
use crate::physics::{util, MetaEvent, PhysicEvent};
use crate::scene::item::{
    apply_scene_item_modifier, apply_scene_item_modifiers, ItemState, SceneItem, SceneItemModifier,
    SceneItemType,
};
use crate::scene::util::{update_background_batch, update_decor_batches};
use crate::ui::vertical_menu::vertical_menu_sprite_info;
use crate::ui::{CursorImmobile, MenuItem};
use crate::ui::{SceneItemPrepareOrder, UiComponent, UserEvent};
use crate::util::velocity_for_behavior;
use crate::{scene, Message, Offset, SceneItemId, ScenePoint, WindowPoint};
use std::ops::Index;

#[derive(PartialEq)]
enum DebugTerrain {
    None,
    Tiles,
    Opacity,
}

pub enum MainStateModifier {
    ChangeSceneItemGridPosition(SceneItemId, GridPoint, GridPoint),
}

pub struct MainState {
    // time
    frame_i: u32,
    start: Instant,

    // map
    pub map: Map,

    // display
    debug: bool,
    debug_terrain: DebugTerrain,
    hide_decor: bool,
    display_offset: Offset,
    sprite_sheet_batch: graphics::spritebatch::SpriteBatch,
    background_batch: graphics::spritebatch::SpriteBatch,
    interiors_batch: graphics::spritebatch::SpriteBatch,
    ui_batch: graphics::spritebatch::SpriteBatch,
    debug_terrain_batch: graphics::spritebatch::SpriteBatch,
    debug_terrain_opacity_mesh_builder: MeshBuilder,
    decor_batches: Vec<graphics::spritebatch::SpriteBatch>,

    // scene items
    scene_items: Vec<SceneItem>,
    scene_items_by_grid_position: HashMap<GridPoint, Vec<usize>>,

    // events
    physics_events: Vec<PhysicEvent>,

    // user interactions
    last_key_consumed: HashMap<KeyCode, Instant>,
    left_click_down: Option<WindowPoint>,
    right_click_down: Option<WindowPoint>,
    current_cursor_point: WindowPoint,
    current_cursor_grid_point: GridPoint,
    cursor_on_same_grid_point_since: Instant,
    waiting_cursor: Vec<CursorImmobile>,
    user_events: Vec<UserEvent>,
    selected_scene_items: Vec<usize>,             // scene_item usize
    scene_item_menu: Option<(usize, ScenePoint)>, // scene_item usize, display_at
    scene_item_prepare_order: Option<SceneItemPrepareOrder>,
    current_prepare_move_found_paths: HashMap<SceneItemId, Vec<GridPoint>>,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<MainState> {
        let map = Map::new(&Path::new("resources/map1.tmx"))?;

        let sprite_sheet_image = graphics::Image::new(ctx, "/sprite_sheet.png")?;
        let sprite_sheet_batch = graphics::spritebatch::SpriteBatch::new(sprite_sheet_image);

        let background_image = graphics::Image::new(
            ctx,
            &Path::new(&format!("/{}", &map.background_image.source)),
        )?;
        let mut background_batch = graphics::spritebatch::SpriteBatch::new(background_image);
        update_background_batch(&mut background_batch);

        let interiors_image = graphics::Image::new(
            ctx,
            &Path::new(&format!("/{}", &map.interiors_image.source)),
        )?;
        let interiors_batch = graphics::spritebatch::SpriteBatch::new(interiors_image);

        let ui_image = graphics::Image::new(ctx, "/ui.png")?;
        let ui_batch = graphics::spritebatch::SpriteBatch::new(ui_image);

        let terrain_image = graphics::Image::new(ctx, format!("/{}", map.terrain.image.source))?;
        let mut debug_terrain_batch = graphics::spritebatch::SpriteBatch::new(terrain_image);
        debug_terrain_batch = scene::util::update_terrain_batch(debug_terrain_batch, &map);
        let debug_terrain_opacity_mesh_builder =
            scene::util::create_debug_terrain_opacity_mesh_builder(&map)?;

        let mut decor_batches = vec![];
        for decor_tileset in map.decor.tilesets.iter() {
            let decor_tiled_image = extract_image_from_tileset(decor_tileset)?;
            let decor_image = graphics::Image::new(ctx, format!("/{}", decor_tiled_image.source))?;
            let batch = graphics::spritebatch::SpriteBatch::new(decor_image);
            decor_batches.push(batch);
        }
        update_decor_batches(&mut decor_batches, &map);

        let mut scene_items = vec![];
        for x in 0..1 {
            for y in 0..4 {
                scene_items.push(SceneItem::new(
                    SceneItemType::Soldier,
                    ScenePoint::new((x as f32 * 24.0) + 100.0, (y as f32 * 24.0) + 100.0),
                    ItemState::new(ItemBehavior::Standing),
                    &map,
                ));
            }
        }

        let mut scene_items_by_grid_position: HashMap<GridPoint, Vec<usize>> = HashMap::new();
        for (i, scene_item) in scene_items.iter().enumerate() {
            let grid_position = util::grid_point_from_scene_point(&scene_item.position, &map);
            scene_items_by_grid_position
                .entry(grid_position)
                .or_default()
                .push(i);
        }

        let main_state = MainState {
            frame_i: 0,
            start: Instant::now(),
            map,
            debug: false,
            debug_terrain: DebugTerrain::None,
            hide_decor: false,
            display_offset: Offset::new(0.0, 0.0),
            sprite_sheet_batch,
            background_batch,
            interiors_batch,
            ui_batch,
            debug_terrain_batch,
            debug_terrain_opacity_mesh_builder,
            decor_batches,
            scene_items,
            scene_items_by_grid_position,
            physics_events: vec![],
            last_key_consumed: HashMap::new(),
            left_click_down: None,
            right_click_down: None,
            current_cursor_point: WindowPoint::new(0.0, 0.0),
            current_cursor_grid_point: GridPoint::new(0, 0),
            cursor_on_same_grid_point_since: Instant::now(),
            waiting_cursor: vec![],
            user_events: vec![],
            selected_scene_items: vec![],
            scene_item_menu: None,
            scene_item_prepare_order: None,
            current_prepare_move_found_paths: HashMap::new(),
        };

        Ok(main_state)
    }

    fn get_scene_item(&self, index: usize) -> &SceneItem {
        self.scene_items
            .get(index)
            .expect(SCENE_ITEMS_CHANGE_ERR_MSG)
    }

    fn get_scene_item_mut(&mut self, index: usize) -> &mut SceneItem {
        self.scene_items
            .get_mut(index)
            .expect(SCENE_ITEMS_CHANGE_ERR_MSG)
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

        if input::keyboard::is_key_pressed(ctx, KeyCode::F12) {
            if self
                .last_key_consumed
                .get(&KeyCode::F12)
                .unwrap_or(&self.start)
                .elapsed()
                .as_millis()
                > 250
            {
                self.debug = !self.debug;
                self.last_key_consumed.insert(KeyCode::F12, Instant::now());
            }
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::F10) {
            if self
                .last_key_consumed
                .get(&KeyCode::F10)
                .unwrap_or(&self.start)
                .elapsed()
                .as_millis()
                > 250
            {
                self.debug_terrain = match &self.debug_terrain {
                    DebugTerrain::None => DebugTerrain::Tiles,
                    DebugTerrain::Tiles => DebugTerrain::Opacity,
                    DebugTerrain::Opacity => DebugTerrain::None,
                };
                self.last_key_consumed.insert(KeyCode::F10, Instant::now());
            }
        }
        if input::keyboard::is_key_pressed(ctx, KeyCode::T) {
            if self
                .last_key_consumed
                .get(&KeyCode::T)
                .unwrap_or(&self.start)
                .elapsed()
                .as_millis()
                > 250
            {
                self.hide_decor = !self.hide_decor;
                self.last_key_consumed.insert(KeyCode::T, Instant::now());
            }
        }

        while let Some(user_event) = self.user_events.pop() {
            match user_event {
                UserEvent::Click(window_click_point) => self.digest_click(window_click_point),
                UserEvent::AreaSelection(window_from, window_to) => {
                    self.digest_area_selection(window_from, window_to)
                }
                UserEvent::RightClick(window_right_click_point) => {
                    self.digest_right_click(window_right_click_point)
                }
                UserEvent::DrawMovePaths => {
                    if let Some(SceneItemPrepareOrder::Move(_))
                    | Some(SceneItemPrepareOrder::MoveFast(_))
                    | Some(SceneItemPrepareOrder::Hide(_)) = &self.scene_item_prepare_order
                    {
                        for scene_item_i in self.selected_scene_items.iter() {
                            if let None = self.current_prepare_move_found_paths.get(scene_item_i) {
                                let scene_item = self.get_scene_item(*scene_item_i);
                                self.current_prepare_move_found_paths.insert(
                                    *scene_item_i,
                                    find_path(
                                        &self.map,
                                        &scene_item.grid_position,
                                        &self.current_cursor_grid_point,
                                    )
                                    .unwrap_or(vec![]),
                                );
                            }
                        }
                    }
                }
                UserEvent::CursorMove(_) => {
                    if let Some(SceneItemPrepareOrder::Move(_))
                    | Some(SceneItemPrepareOrder::MoveFast(_))
                    | Some(SceneItemPrepareOrder::Hide(_)) = &self.scene_item_prepare_order
                    {
                        let waiting_cursor_not_move =
                            CursorImmobile(Duration::from_millis(250), UserEvent::DrawMovePaths);
                        if !self.waiting_cursor.contains(&waiting_cursor_not_move) {
                            self.waiting_cursor.push(waiting_cursor_not_move);
                        }
                    }
                }
            }
        }

        // Check waiting cursor immobile instructions
        let cursor_immobile_since = self.cursor_on_same_grid_point_since.elapsed();
        let mut re_push: Vec<CursorImmobile> = vec![];
        while let Some(waiting_cursor) = self.waiting_cursor.pop() {
            if cursor_immobile_since >= waiting_cursor.0 {
                self.user_events.push(waiting_cursor.1.clone());
            } else {
                re_push.push(waiting_cursor)
            }
        }
        self.waiting_cursor.extend(re_push);
    }

    fn digest_click(&mut self, window_click_point: WindowPoint) {
        let scene_click_point =
            scene_point_from_window_point(&window_click_point, &self.display_offset);
        let mut scene_item_selected = false;
        let mut scene_item_menu_clicked = false;
        let mut prepare_order_clicked = false;

        // Click on scene item
        if let Some(scene_item_usize) =
            self.get_first_scene_item_for_scene_point(&scene_click_point)
        {
            self.selected_scene_items.drain(..);
            self.selected_scene_items.push(scene_item_usize);
            scene_item_selected = true;
        }

        // Click during preparing order
        if let Some(scene_item_prepare_order) = &self.scene_item_prepare_order {
            match scene_item_prepare_order {
                SceneItemPrepareOrder::Move(scene_item_usize)
                | SceneItemPrepareOrder::MoveFast(scene_item_usize)
                | SceneItemPrepareOrder::Hide(scene_item_usize) => {
                    let order = match scene_item_prepare_order {
                        SceneItemPrepareOrder::Move(_) => Order::MoveTo(scene_click_point),
                        SceneItemPrepareOrder::MoveFast(_) => Order::MoveFastTo(scene_click_point),
                        SceneItemPrepareOrder::Hide(_) => Order::HideTo(scene_click_point),
                    };
                    let mut scene_item = self.get_scene_item_mut(*scene_item_usize);
                    scene_item.next_order = Some(order);
                    self.current_prepare_move_found_paths = HashMap::new();
                }
            }

            self.scene_item_prepare_order = None;
            prepare_order_clicked = true;
        }

        // Click during display of scene item menu
        if let Some((scene_item_usize, scene_menu_point)) = self.scene_item_menu {
            let menu_sprite_info = vertical_menu_sprite_info(UiComponent::SceneItemMenu);
            if let Some(menu_item) =
                menu_sprite_info.item_clicked(&scene_menu_point, &scene_click_point)
            {
                match menu_item {
                    MenuItem::Move => {
                        self.scene_item_prepare_order =
                            Some(SceneItemPrepareOrder::Move(scene_item_usize));
                        self.scene_item_menu = None;
                    }
                    MenuItem::MoveFast => {
                        self.scene_item_prepare_order =
                            Some(SceneItemPrepareOrder::MoveFast(scene_item_usize));
                        self.scene_item_menu = None;
                    }
                    MenuItem::Hide => {
                        self.scene_item_prepare_order =
                            Some(SceneItemPrepareOrder::Hide(scene_item_usize));
                        self.scene_item_menu = None;
                    }
                }
            };
            self.scene_item_menu = None;
            scene_item_menu_clicked = true;
        };

        if !prepare_order_clicked && !scene_item_menu_clicked && !scene_item_selected {
            self.selected_scene_items.drain(..);
        }
    }

    fn digest_right_click(&mut self, window_right_click_point: WindowPoint) {
        let scene_right_click_point =
            scene_point_from_window_point(&window_right_click_point, &self.display_offset);

        // TODO: aucune selection et right click sur un item: scene_item_menu sur un item
        // TODO: selection et right click sur un item de la selection: scene_item_menu sur un TOUS les item de la selection
        // TODO: selection et right click sur un item PAS dans la selection: scene_item_menu sur un item

        if let Some(scene_item_usize) =
            self.get_first_scene_item_for_scene_point(&scene_right_click_point)
        {
            if self.selected_scene_items.contains(&scene_item_usize) {
                let scene_item = self.get_scene_item(scene_item_usize);
                self.scene_item_menu = Some((scene_item_usize, scene_item.position))
            }
        }
    }

    fn digest_area_selection(&mut self, window_from: WindowPoint, window_to: WindowPoint) {
        let scene_from = scene_point_from_window_point(&window_from, &self.display_offset);
        let scene_to = scene_point_from_window_point(&window_to, &self.display_offset);
        self.selected_scene_items.drain(..);
        self.selected_scene_items
            .extend(self.get_scene_items_for_scene_area(&scene_from, &scene_to));
    }

    fn change_scene_item_grid_position(
        &mut self,
        scene_item_i: usize,
        from_grid_position: GridPoint,
        to_grid_position: GridPoint,
    ) {
        let grid_position_scene_items = self
            .scene_items_by_grid_position
            .get_mut(&from_grid_position)
            .expect("Scene item should be here !");
        let x = grid_position_scene_items
            .iter()
            .position(|x| *x == scene_item_i)
            .expect("Scene item should be here !");
        grid_position_scene_items.remove(x);
        self.scene_items_by_grid_position
            .entry(to_grid_position)
            .or_default()
            .push(scene_item_i)
    }

    // TODO: manage errors
    fn physics(&mut self) {
        let mut messages: Vec<Message> = vec![];

        // Scene items movements
        for (scene_item_i, scene_item) in self.scene_items.iter().enumerate() {
            messages.extend(produce_physics_messages_for_scene_item(
                scene_item_i,
                &scene_item,
                &self.map,
            ))
        }

        // (FAKE) Drop a bomb to motivate stop move
        if self.frame_i % 600 == 0 && self.frame_i != 0 {
            self.physics_events.push(PhysicEvent::Explosion);
        }

        self.consume_messages(messages);
    }

    fn consume_messages(&mut self, messages: Vec<Message>) {
        for message in messages.into_iter() {
            match message {
                Message::SceneItemMessage(i, scene_item_modifier) => {
                    let scene_item = self.get_scene_item_mut(i);
                    apply_scene_item_modifier(scene_item, scene_item_modifier);
                }
                Message::MainStateMessage(main_state_modifier) => match main_state_modifier {
                    MainStateModifier::ChangeSceneItemGridPosition(
                        scene_item_i,
                        from_grid_position,
                        to_grid_position,
                    ) => {
                        self.change_scene_item_grid_position(
                            scene_item_i,
                            from_grid_position,
                            to_grid_position,
                        );
                    }
                },
            }
        }
    }

    fn metas(&mut self) {
        for physic_event in &self.physics_events {
            match physic_event {
                PhysicEvent::Explosion => {
                    for scene_item in self.scene_items.iter_mut() {
                        scene_item.meta_events.push(MetaEvent::FeelExplosion);
                    }
                }
            }
        }
    }

    fn seek(&mut self) {
        let mut messages: Vec<Message> = vec![];

        for (scene_item_from_i, scene_item_from) in self.scene_items.iter().enumerate() {
            for (scene_item_to_i, scene_item_to) in self.scene_items.iter().enumerate() {
                if scene_item_from_i == scene_item_to_i {
                    continue;
                }
                // Use bresenham algorithm to compute opacity etc ...
            }
        }

        self.consume_messages(messages)
    }

    fn animate(&mut self) {
        let mut messages: Vec<Message> = vec![];

        for (_, scene_item) in self.scene_items.iter_mut().enumerate() {
            messages.extend(apply_scene_item_modifiers(
                scene_item,
                digest_next_order(&scene_item, &self.map),
            ));
            messages.extend(apply_scene_item_modifiers(
                scene_item,
                digest_current_order(&scene_item, &self.map),
            ));
            messages.extend(apply_scene_item_modifiers(
                scene_item,
                digest_current_behavior(&scene_item, &self.map),
            ));
        }

        self.consume_messages(messages);
    }

    fn tick_sprites(&mut self) {
        for scene_item in self.scene_items.iter_mut() {
            scene_item.tick_sprite();
        }
    }

    fn get_first_scene_item_for_scene_point(&self, scene_position: &ScenePoint) -> Option<usize> {
        // TODO: if found multiple: select nearest
        for (i, scene_item) in self.scene_items.iter().enumerate() {
            let sprite_info = scene_item.sprite_info();
            if scene_item.position.x >= scene_position.x - sprite_info.tile_width
                && scene_item.position.x <= scene_position.x + sprite_info.tile_width
                && scene_item.position.y >= scene_position.y - sprite_info.tile_height
                && scene_item.position.y <= scene_position.y + sprite_info.tile_height
            {
                return Some(i);
            }
        }

        None
    }

    fn get_scene_items_for_scene_area(&self, from: &ScenePoint, to: &ScenePoint) -> Vec<usize> {
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

    fn generate_scene_item_sprites(&mut self) -> GameResult {
        for scene_item in self.scene_items.iter() {
            self.sprite_sheet_batch.add(
                scene_item
                    .as_draw_param(scene_item.current_frame)
                    .dest(scene_item.position.clone()),
            );
        }

        Ok(())
    }

    fn generate_scene_item_menu_sprites(&mut self) -> GameResult {
        if let Some((_, scene_point)) = self.scene_item_menu {
            for draw_param in vertical_menu_sprite_info(UiComponent::SceneItemMenu)
                .as_draw_params(&scene_point, &self.current_cursor_point)
            {
                self.ui_batch.add(draw_param);
            }
        }

        Ok(())
    }

    fn update_interior_sprites(&mut self) -> GameResult {
        self.interiors_batch.clear();
        for interior in self.map.interiors_objects.objects.iter() {
            let start_x = interior.x;
            let start_y = interior.y;
            let end_x = start_x + interior.width;
            let end_y = start_y + interior.height;

            for scene_item in self.scene_items.iter() {
                if scene_item.position.x >= start_x
                    && scene_item.position.x <= end_x
                    && scene_item.position.y >= start_y
                    && scene_item.position.y <= end_y
                {
                    self.interiors_batch.add(
                        graphics::DrawParam::new()
                            .src(graphics::Rect::new(
                                start_x / self.map.interiors_image.width as f32,
                                start_y / self.map.interiors_image.height as f32,
                                interior.width / self.map.interiors_image.width as f32,
                                interior.height / self.map.interiors_image.height as f32,
                            ))
                            .dest(ScenePoint::new(start_x, start_y)),
                    );
                    continue;
                }
            }
        }

        Ok(())
    }

    fn update_mesh_builder_with_debug(
        &self,
        mut mesh_builder: MeshBuilder,
    ) -> GameResult<MeshBuilder> {
        if self.debug {
            // Draw circle on each scene item position
            for scene_item in self.scene_items.iter() {
                mesh_builder.circle(
                    DrawMode::fill(),
                    scene_item.position.clone(),
                    2.0,
                    2.0,
                    graphics::WHITE,
                )?;
            }

            // Draw circle where left click down
            if let Some(window_left_click_down_point) = self.left_click_down {
                let scene_left_click_down_point = scene_point_from_window_point(
                    &window_left_click_down_point,
                    &self.display_offset,
                );
                mesh_builder.circle(
                    DrawMode::fill(),
                    scene_left_click_down_point,
                    2.0,
                    2.0,
                    graphics::YELLOW,
                )?;
            }

            // Draw circle at cursor position
            mesh_builder.circle(
                DrawMode::fill(),
                scene_point_from_window_point(&self.current_cursor_point, &self.display_offset),
                2.0,
                2.0,
                graphics::BLUE,
            )?;
        }

        GameResult::Ok(mesh_builder)
    }

    fn update_mesh_builder_with_selected_items(
        &self,
        mut mesh_builder: MeshBuilder,
    ) -> GameResult<MeshBuilder> {
        for i in &self.selected_scene_items {
            let scene_item = self.get_scene_item(*i);

            // Selection square
            mesh_builder.rectangle(
                DrawMode::Stroke(StrokeOptions::default()),
                graphics::Rect::new(
                    scene_item.position.x - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                    scene_item.position.y - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                    DEFAULT_SELECTED_SQUARE_SIDE,
                    DEFAULT_SELECTED_SQUARE_SIDE,
                ),
                graphics::GREEN,
            )?;

            // Move path if moving
            match &scene_item.state.current_behavior {
                ItemBehavior::Standing => {}
                ItemBehavior::HideTo(_, grid_path)
                | ItemBehavior::MoveTo(_, grid_path)
                | ItemBehavior::MoveFastTo(_, grid_path) => {
                    let mut points = vec![scene_item.position];
                    for grid_point in grid_path.iter() {
                        points.push(scene_point_from_grid_point(grid_point, &self.map))
                    }

                    mesh_builder.line(
                        &points,
                        2.0,
                        Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 0.2,
                        },
                    )?;
                }
            }
        }

        GameResult::Ok(mesh_builder)
    }

    fn update_mesh_builder_with_selection_area(
        &self,
        mut mesh_builder: MeshBuilder,
    ) -> GameResult<MeshBuilder> {
        if let Some(window_left_click_down_point) = self.left_click_down {
            let scene_left_click_down_point =
                scene_point_from_window_point(&window_left_click_down_point, &self.display_offset);
            let scene_current_cursor_position =
                scene_point_from_window_point(&self.current_cursor_point, &self.display_offset);
            if scene_left_click_down_point != scene_current_cursor_position {
                mesh_builder.rectangle(
                    DrawMode::stroke(1.0),
                    graphics::Rect::new(
                        scene_left_click_down_point.x,
                        scene_left_click_down_point.y,
                        scene_current_cursor_position.x - scene_left_click_down_point.x,
                        scene_current_cursor_position.y - scene_left_click_down_point.y,
                    ),
                    graphics::GREEN,
                )?;
            }
        }

        GameResult::Ok(mesh_builder)
    }

    fn update_mesh_builder_with_prepare_order(
        &self,
        mut mesh_builder: MeshBuilder,
    ) -> GameResult<MeshBuilder> {
        if let Some(scene_item_prepare_order) = &self.scene_item_prepare_order {
            match scene_item_prepare_order {
                SceneItemPrepareOrder::Move(scene_item_usize)
                | SceneItemPrepareOrder::MoveFast(scene_item_usize)
                | SceneItemPrepareOrder::Hide(scene_item_usize) => {
                    let color = match &scene_item_prepare_order {
                        SceneItemPrepareOrder::Move(_) => graphics::BLUE,
                        SceneItemPrepareOrder::MoveFast(_) => graphics::MAGENTA,
                        SceneItemPrepareOrder::Hide(_) => graphics::YELLOW,
                    };

                    let scene_item = self.get_scene_item(*scene_item_usize);
                    mesh_builder.line(
                        &vec![
                            scene_item.position.clone(),
                            scene_point_from_window_point(
                                &self.current_cursor_point,
                                &self.display_offset,
                            ),
                        ],
                        2.0,
                        color,
                    )?;
                }
            }
        }

        for (scene_item_i, path) in self.current_prepare_move_found_paths.iter() {
            let scene_item = self.get_scene_item(*scene_item_i);
            let mut points = vec![scene_item.position];
            for scene_grid_point in path.iter() {
                points.push(scene_point_from_grid_point(scene_grid_point, &self.map))
            }
            points.push(scene_point_from_window_point(
                &self.current_cursor_point,
                &self.display_offset,
            ));

            mesh_builder.line(
                &points,
                2.0,
                Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 0.2,
                },
            )?;
        }

        GameResult::Ok(mesh_builder)
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
            let tick_seek = self.frame_i % SEEK_EACH == 0;
            let tick_physics = self.frame_i % PHYSICS_EACH == 0;
            let tick_meta = self.frame_i % META_EACH == 0;
            let tick_interiors = self.frame_i % INTERIORS_EACH == 0;

            // Apply moves, explosions, etc
            if tick_physics {
                self.physics();
            }

            // Generate meta events according to physics events and current physic state
            if tick_meta {
                self.metas();
            }

            // Seek scene items between them
            if tick_seek {
                self.seek();
            }

            // Animate scene items according to meta events
            if tick_animate {
                self.animate();
            };

            // Change scene items tiles
            if tick_sprite {
                self.tick_sprites();
            }

            // Compute interiors sprites
            if tick_interiors {
                self.update_interior_sprites()?;
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

        self.generate_scene_item_sprites()?;
        self.generate_scene_item_menu_sprites()?;

        scene_mesh_builder = self.update_mesh_builder_with_debug(scene_mesh_builder)?;
        scene_mesh_builder = self.update_mesh_builder_with_selected_items(scene_mesh_builder)?;
        scene_mesh_builder = self.update_mesh_builder_with_selection_area(scene_mesh_builder)?;
        scene_mesh_builder = self.update_mesh_builder_with_prepare_order(scene_mesh_builder)?;

        let window_draw_param = graphics::DrawParam::new().dest(window_point_from_scene_point(
            &ScenePoint::new(0.0, 0.0),
            &self.display_offset,
        ));

        // Draw map background
        graphics::draw(ctx, &self.background_batch, window_draw_param)?;

        // Draw interiors
        graphics::draw(ctx, &self.interiors_batch, window_draw_param)?;

        // Draw terrain debug
        if self.debug_terrain == DebugTerrain::Tiles {
            graphics::draw(ctx, &self.debug_terrain_batch, window_draw_param)?;
        } else if self.debug_terrain == DebugTerrain::Opacity {
            let debug_terrain_opacity_mesh =
                self.debug_terrain_opacity_mesh_builder.build(ctx).unwrap();
            graphics::draw(ctx, &debug_terrain_opacity_mesh, window_draw_param)?;
        }

        // Draw scene items
        graphics::draw(ctx, &self.sprite_sheet_batch, window_draw_param)?;

        // Draw decor
        if !self.hide_decor {
            for decor_batch in self.decor_batches.iter() {
                graphics::draw(ctx, decor_batch, window_draw_param)?;
            }
        }

        // Draw user interactions
        if let Ok(scene_mesh) = scene_mesh_builder.build(ctx) {
            graphics::draw(ctx, &scene_mesh, window_draw_param)?;
        }

        // Draw ui
        graphics::draw(ctx, &self.ui_batch, window_draw_param)?;

        self.sprite_sheet_batch.clear();
        self.ui_batch.clear();

        graphics::present(ctx)?;

        // println!("FPS: {}", ggez::timer::fps(ctx));
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => {
                self.left_click_down = Some(WindowPoint::new(x, y));
            }
            MouseButton::Right => {
                self.right_click_down = Some(WindowPoint::new(x, y));
            }
            MouseButton::Middle => {}
            MouseButton::Other(_) => {}
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => {
                if let Some(left_click_down) = self.left_click_down {
                    if left_click_down == WindowPoint::new(x, y) {
                        self.user_events.push(UserEvent::Click(left_click_down));
                    } else {
                        let from = WindowPoint::new(
                            cmp::min(left_click_down.x as i32, x as i32) as f32,
                            cmp::min(left_click_down.y as i32, y as i32) as f32,
                        );
                        let to = WindowPoint::new(
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
        let new_current_cursor_position = WindowPoint::new(x, y);
        let new_current_grid_cursor_position = grid_point_from_scene_point(
            &scene_point_from_window_point(&new_current_cursor_position, &self.display_offset),
            &self.map,
        );

        if self.current_cursor_point != new_current_cursor_position {
            self.user_events
                .push(UserEvent::CursorMove(new_current_cursor_position));
            self.current_cursor_point = new_current_cursor_position;
        };

        if self.current_cursor_grid_point != new_current_grid_cursor_position {
            self.current_cursor_grid_point = new_current_grid_cursor_position;
            self.current_prepare_move_found_paths = HashMap::new();
            self.cursor_on_same_grid_point_since = Instant::now();
        } else {
            self.cursor_on_same_grid_point_since = Instant::now();
        }
    }
}
