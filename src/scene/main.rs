use std::cmp;
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use ggez::event::MouseButton;
use ggez::graphics::{Color, DrawMode, FilterMode, MeshBuilder, StrokeOptions, Text};
use ggez::input::keyboard::KeyCode;
use ggez::timer::check_update_time;
use ggez::{event, graphics, input, Context, GameResult};

use crate::audio::{Audio, Sound};
use crate::behavior::animate::{digest_behavior, digest_current_order, digest_next_order};
use crate::behavior::order::Order;
use crate::behavior::ItemBehavior;
use crate::config::{
    ANIMATE_EACH, DEFAULT_SELECTED_SQUARE_SIDE, DEFAULT_SELECTED_SQUARE_SIDE_HALF,
    DISPLAY_OFFSET_BY, DISPLAY_OFFSET_BY_SPEED, INTERIORS_EACH, MAX_FRAME_I, PHYSICS_EACH,
    SCENE_ITEMS_CHANGE_ERR_MSG, SEEK_EACH, SPRITE_EACH, TARGET_FPS,
};
use crate::gameplay::weapon::{Weapon, WeaponType};
use crate::map::util::extract_image_from_tileset;
use crate::map::Map;
use crate::physics::item::produce_physics_messages_for_scene_item;
use crate::physics::path::find_path;
use crate::physics::projectile::{bullet_fire, Projectile};
use crate::physics::util::{grid_point_from_scene_point, scene_point_from_window_point};
use crate::physics::util::{scene_point_from_grid_point, window_point_from_scene_point};
use crate::physics::visibility::Visibility;
use crate::physics::GridPoint;
use crate::physics::{util, MetaEvent, PhysicEvent};
use crate::scene::item::{
    apply_scene_item_modifier, apply_scene_item_modifiers, SceneItem, SceneItemModifier,
    SceneItemType, Side,
};
use crate::scene::util::{incapacitated, update_background_batch, update_decor_batches};
use crate::ui::vertical_menu::vertical_menu_sprite_info;
use crate::ui::{CursorImmobile, MenuItem};
use crate::ui::{SceneItemPrepareOrder, UiComponent, UserEvent};
use crate::{scene, FrameI, Message, Offset, SceneItemId, ScenePoint, WindowPoint};
use std::io::BufReader;

#[derive(PartialEq)]
enum DebugTerrain {
    None,
    Tiles,
    Opacity,
}

pub enum MainStateModifier {
    ChangeSceneItemGridPosition(SceneItemId, GridPoint, GridPoint),
    InsertCurrentPrepareMoveFoundPaths(SceneItemId, Vec<GridPoint>),
    NewSeenOpponent(SceneItemId),
    LostSeenOpponent(SceneItemId),
    PushPhysicEvent(PhysicEvent),
    NewProjectile(Projectile),
    NewSound(Sound),
}

pub struct MainState {
    // time
    frame_i: FrameI, // FIXME BS NOW: regler le probleme du reset (acquiring_until etc)
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
    projectiles: Vec<Projectile>,

    // scene items
    scene_items: Vec<SceneItem>,
    scene_items_by_grid_position: HashMap<GridPoint, Vec<SceneItemId>>,
    scene_items_by_side: HashMap<Side, Vec<SceneItemId>>,

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
    selected_scene_items: Vec<SceneItemId>, // scene_item usize
    scene_item_menu: Option<(SceneItemId, ScenePoint)>, // scene_item usize, display_at
    scene_item_prepare_order: Option<SceneItemPrepareOrder>,
    current_prepare_move_found_paths: HashMap<SceneItemId, Vec<GridPoint>>,

    // Gameplay
    current_side: Side,
    opposite_visible_scene_items: Vec<SceneItemId>,

    audio: Audio,
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
        let mut scene_items_by_grid_position: HashMap<GridPoint, Vec<usize>> = HashMap::new();
        let mut scene_items_by_side: HashMap<Side, Vec<SceneItemId>> = HashMap::new();
        let mut scene_item_id: usize = 0;

        for x in 0..1 {
            for y in 0..5 {
                let scene_item = SceneItem::new(
                    scene_item_id,
                    SceneItemType::Soldier,
                    ScenePoint::new((x as f32 * 24.0) + 100.0, (y as f32 * 24.0) + 100.0),
                    ItemBehavior::Standing,
                    &map,
                    Side::A,
                    Weapon::new(WeaponType::GarandM1),
                );
                scene_items_by_side
                    .entry(Side::A)
                    .or_default()
                    .push(scene_item_id);
                let grid_position = util::grid_point_from_scene_point(&scene_item.position, &map);
                scene_items_by_grid_position
                    .entry(grid_position)
                    .or_default()
                    .push(scene_item_id);
                scene_items.push(scene_item);
                scene_item_id += 1;
            }
        }

        for x in 0..1 {
            for y in 0..5 {
                let scene_item = SceneItem::new(
                    scene_item_id,
                    SceneItemType::Soldier,
                    ScenePoint::new((x as f32 * 24.0) + 550.0, (y as f32 * 24.0) + 200.0),
                    ItemBehavior::Standing,
                    &map,
                    Side::B,
                    Weapon::new(WeaponType::MauserG41),
                );
                scene_items_by_side
                    .entry(Side::B)
                    .or_default()
                    .push(scene_item_id);
                let grid_position = util::grid_point_from_scene_point(&scene_item.position, &map);
                scene_items_by_grid_position
                    .entry(grid_position)
                    .or_default()
                    .push(scene_item_id);
                scene_items.push(scene_item);
                scene_item_id += 1;
            }
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
            projectiles: vec![],
            scene_items,
            scene_items_by_grid_position,
            scene_items_by_side,
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
            current_side: Side::A,
            opposite_visible_scene_items: vec![],
            audio: Audio::new(),
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

    fn inputs(&mut self, ctx: &Context) -> Vec<Message> {
        let mut messages = vec![];

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
        if input::keyboard::is_key_pressed(ctx, KeyCode::F9) {
            if self
                .last_key_consumed
                .get(&KeyCode::F9)
                .unwrap_or(&self.start)
                .elapsed()
                .as_millis()
                > 250
            {
                self.current_side = match self.current_side {
                    Side::A => Side::B,
                    Side::B => Side::A,
                };
                self.last_key_consumed.insert(KeyCode::F9, Instant::now());
                self.selected_scene_items = vec![];
                self.opposite_visible_scene_items = vec![];
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
                UserEvent::Click(window_click_point) => {
                    messages.extend(self.digest_click(window_click_point))
                }
                UserEvent::AreaSelection(window_from, window_to) => {
                    messages.extend(self.digest_area_selection(window_from, window_to))
                }
                UserEvent::RightClick(window_right_click_point) => {
                    messages.extend(self.digest_right_click(window_right_click_point))
                }
                UserEvent::DrawMovePaths => {
                    if let Some(SceneItemPrepareOrder::Move(_))
                    | Some(SceneItemPrepareOrder::MoveFast(_))
                    | Some(SceneItemPrepareOrder::Hide(_)) = &self.scene_item_prepare_order
                    {
                        for scene_item_i in self.selected_scene_items.iter() {
                            if let None = self.current_prepare_move_found_paths.get(scene_item_i) {
                                let scene_item = self.get_scene_item(*scene_item_i);
                                messages.push(Message::MainStateMessage(
                                    MainStateModifier::InsertCurrentPrepareMoveFoundPaths(
                                        *scene_item_i,
                                        find_path(
                                            &self.map,
                                            &scene_item.grid_position,
                                            &self.current_cursor_grid_point,
                                        )
                                        .unwrap_or(vec![]),
                                    ),
                                ));
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

        messages
    }

    fn digest_click(&mut self, window_click_point: WindowPoint) -> Vec<Message> {
        let mut messages = vec![];

        let scene_click_point =
            scene_point_from_window_point(&window_click_point, &self.display_offset);
        let mut scene_item_selected = false;
        let mut scene_item_menu_clicked = false;
        let mut prepare_order_clicked = false;

        // Click on scene item
        if let Some(scene_item_usize) =
            self.get_first_scene_item_for_scene_point(&scene_click_point, true)
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
                    messages.push(Message::SceneItemMessage(
                        *scene_item_usize,
                        SceneItemModifier::SetNextOrder(order),
                    ));
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
        };

        messages
    }

    fn digest_right_click(&mut self, window_right_click_point: WindowPoint) -> Vec<Message> {
        let scene_right_click_point =
            scene_point_from_window_point(&window_right_click_point, &self.display_offset);

        // TODO: aucune selection et right click sur un item: scene_item_menu sur un item
        // TODO: selection et right click sur un item de la selection: scene_item_menu sur un TOUS les item de la selection
        // TODO: selection et right click sur un item PAS dans la selection: scene_item_menu sur un item

        if let Some(scene_item_usize) =
            self.get_first_scene_item_for_scene_point(&scene_right_click_point, true)
        {
            if self.selected_scene_items.contains(&scene_item_usize) {
                let scene_item = self.get_scene_item(scene_item_usize);
                self.scene_item_menu = Some((scene_item_usize, scene_item.position))
            }
        };

        vec![]
    }

    fn digest_area_selection(
        &mut self,
        window_from: WindowPoint,
        window_to: WindowPoint,
    ) -> Vec<Message> {
        let scene_from = scene_point_from_window_point(&window_from, &self.display_offset);
        let scene_to = scene_point_from_window_point(&window_to, &self.display_offset);
        self.selected_scene_items.drain(..);
        self.selected_scene_items
            .extend(self.get_scene_items_for_scene_area(&scene_from, &scene_to, true));

        vec![]
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

    fn physics(&mut self) {
        let mut messages: Vec<Message> = vec![];

        while let Some(physic_event) = &self.physics_events.pop() {
            match physic_event {
                PhysicEvent::BulletFire(visibility) => {
                    let from_scene_item = self.get_scene_item(visibility.from_scene_id);
                    let to_scene_item = if let Some(to_scene_item_id) = visibility.to_scene_item_id
                    {
                        Some(self.get_scene_item(to_scene_item_id))
                    } else {
                        None
                    };
                    messages.extend(bullet_fire(
                        self.frame_i,
                        &visibility,
                        from_scene_item,
                        to_scene_item,
                    ));
                }
            }
        }

        // Ex: Scene items movements
        for (scene_item_i, scene_item) in self.scene_items.iter().enumerate() {
            messages.extend(produce_physics_messages_for_scene_item(
                scene_item_i,
                &scene_item,
                &self.map,
            ))
        }

        self.consume_messages(messages);
    }

    fn consume_messages(&mut self, messages: Vec<Message>) {
        let frame_i = self.frame_i;
        for message in messages.into_iter() {
            match message {
                Message::SceneItemMessage(i, scene_item_modifier) => {
                    let scene_item = self.get_scene_item_mut(i);
                    apply_scene_item_modifier(frame_i, scene_item, scene_item_modifier);
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
                    MainStateModifier::InsertCurrentPrepareMoveFoundPaths(scene_item_i, path) => {
                        self.current_prepare_move_found_paths
                            .insert(scene_item_i, path);
                    }
                    MainStateModifier::NewSeenOpponent(scene_item_id) => {
                        self.opposite_visible_scene_items.push(scene_item_id);
                    }
                    MainStateModifier::LostSeenOpponent(scene_item_id) => {
                        self.opposite_visible_scene_items.remove(
                            self.opposite_visible_scene_items
                                .iter()
                                .position(|x| *x == scene_item_id)
                                .expect("Must be here"),
                        );
                    }
                    MainStateModifier::PushPhysicEvent(event) => {
                        self.physics_events.push(event);
                    }
                    MainStateModifier::NewProjectile(projectile) => {
                        self.projectiles.push(projectile);
                    }
                    MainStateModifier::NewSound(sound) => self.audio.play(sound),
                },
            }
        }
    }

    fn seek(&mut self) {
        let mut messages: Vec<Message> = vec![];
        let mut see_opponents: Vec<SceneItemId> = vec![];

        for (scene_item_from_i, scene_item_from) in self.scene_items.iter().enumerate() {
            if incapacitated(scene_item_from) {
                continue;
            }

            let mut visibilities: Vec<Visibility> = vec![];
            for (scene_item_to_i, scene_item_to) in self.scene_items.iter().enumerate() {
                if scene_item_from.side == scene_item_to.side || incapacitated(scene_item_to) {
                    continue;
                }
                let visibility = Visibility::with_scene_item_target(
                    self.frame_i,
                    scene_item_from,
                    &scene_item_to,
                    &self.map,
                );
                if scene_item_to.side != self.current_side {
                    if visibility.visible {
                        if !see_opponents.contains(&scene_item_to_i) {
                            see_opponents.push(scene_item_to_i);
                        }
                    }
                }
                visibilities.push(visibility);
            }
            messages.push(Message::SceneItemMessage(
                scene_item_from_i,
                SceneItemModifier::ChangeVisibilities(visibilities),
            ));
        }

        // Remove opponent not seen anymore
        for previously_seen_opponent in self.opposite_visible_scene_items.iter() {
            if !see_opponents.contains(previously_seen_opponent) {
                messages.push(Message::MainStateMessage(
                    MainStateModifier::LostSeenOpponent(*previously_seen_opponent),
                ));
            }
        }

        // Add new seen opponents
        for see_opponent in see_opponents.iter() {
            if !self.opposite_visible_scene_items.contains(see_opponent) {
                messages.push(Message::MainStateMessage(
                    MainStateModifier::NewSeenOpponent(*see_opponent),
                ));
            }
        }

        self.consume_messages(messages)
    }

    fn animate(&mut self) {
        let mut messages: Vec<Message> = vec![];

        for (_, scene_item) in self.scene_items.iter_mut().enumerate() {
            if incapacitated(scene_item) {
                continue;
            }

            messages.extend(apply_scene_item_modifiers(
                self.frame_i,
                scene_item,
                digest_next_order(&scene_item, &self.map),
            ));
            messages.extend(apply_scene_item_modifiers(
                self.frame_i,
                scene_item,
                digest_current_order(&scene_item, &self.map),
            ));
            messages.extend(apply_scene_item_modifiers(
                self.frame_i,
                scene_item,
                digest_behavior(self.frame_i, &scene_item, &self.map),
            ));
        }

        self.consume_messages(messages);
    }

    fn tick_sprites(&mut self) {
        for scene_item in self.scene_items.iter_mut() {
            scene_item.tick_sprite();
        }
    }

    fn get_first_scene_item_for_scene_point(
        &self,
        scene_position: &ScenePoint,
        limit: bool,
    ) -> Option<usize> {
        // TODO: if found multiple: select nearest
        for (i, scene_item) in self.scene_items.iter().enumerate() {
            if limit
                && scene_item.side != self.current_side
                && !self.opposite_visible_scene_items.contains(&i)
            {
                continue;
            }

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

    fn get_scene_items_for_scene_area(
        &self,
        from: &ScenePoint,
        to: &ScenePoint,
        limit: bool,
    ) -> Vec<usize> {
        let mut selection = vec![];

        for (i, scene_item) in self.scene_items.iter().enumerate() {
            if limit
                && scene_item.side != self.current_side
                && !self.opposite_visible_scene_items.contains(&i)
            {
                continue;
            }

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
        for (i, scene_item) in self.scene_items.iter().enumerate() {
            if scene_item.side != self.current_side
                && !self.opposite_visible_scene_items.contains(&i)
                && !incapacitated(scene_item)
            {
                continue;
            }

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

            for (i, scene_item) in self.scene_items.iter().enumerate() {
                if incapacitated(scene_item) {
                    continue;
                }

                if scene_item.side != self.current_side
                    && !self.opposite_visible_scene_items.contains(&i)
                {
                    continue;
                }

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

    fn update_scene_mesh_with_debug(
        &self,
        mut mesh_builder: MeshBuilder,
    ) -> GameResult<MeshBuilder> {
        if self.debug {
            // Draw circle on each scene item position
            for scene_item in self.scene_items.iter() {
                let color = if scene_item.side == self.current_side {
                    graphics::GREEN
                } else {
                    graphics::RED
                };
                mesh_builder.circle(
                    DrawMode::fill(),
                    scene_item.position.clone(),
                    2.0,
                    2.0,
                    color,
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

    fn update_scene_mesh_with_selected_items(
        &self,
        mut mesh_builder: MeshBuilder,
    ) -> GameResult<MeshBuilder> {
        for i in &self.selected_scene_items {
            let scene_item = self.get_scene_item(*i);
            let color = if scene_item.side == self.current_side {
                graphics::GREEN
            } else {
                graphics::RED
            };

            // Selection square
            mesh_builder.rectangle(
                DrawMode::Stroke(StrokeOptions::default()),
                graphics::Rect::new(
                    scene_item.position.x - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                    scene_item.position.y - DEFAULT_SELECTED_SQUARE_SIDE_HALF,
                    DEFAULT_SELECTED_SQUARE_SIDE,
                    DEFAULT_SELECTED_SQUARE_SIDE,
                ),
                color,
            )?;

            // Move path if moving
            match &scene_item.behavior {
                ItemBehavior::Dead | ItemBehavior::Unconscious => {}
                ItemBehavior::Standing => {}
                ItemBehavior::Hide => {}
                ItemBehavior::EngageSceneItem(_) => {}
                ItemBehavior::EngageGridPoint(_) => {}
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

    fn update_scene_mesh_with_selection_area(
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

    fn update_scene_mesh_with_prepare_order(
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

    fn update_scene_mesh_with_projectiles(
        &mut self,
        mut mesh_builder: MeshBuilder,
    ) -> GameResult<MeshBuilder> {
        let mut continue_projectiles: Vec<Projectile> = vec![];

        while let Some(projectile) = self.projectiles.pop() {
            if projectile.start <= self.frame_i && projectile.end > self.frame_i {
                let color = if projectile.side == self.current_side {
                    Color {
                        r: 0.0,
                        g: 1.0,
                        b: 0.0,
                        a: 0.5,
                    }
                } else {
                    Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.5,
                    }
                };

                mesh_builder.line(
                    &vec![projectile.from_scene_point, projectile.to_scene_point],
                    1.0,
                    color,
                )?;
            }

            // Projectile must be displayed net frame too
            if projectile.end >= self.frame_i {
                continue_projectiles.push(projectile)
            }
        }

        self.projectiles = continue_projectiles;
        GameResult::Ok(mesh_builder)
    }

    fn update_visibilities_mesh(&self, mut mesh_builder: MeshBuilder) -> GameResult<MeshBuilder> {
        if self.debug {
            for selected_scene_item_i in self.selected_scene_items.iter() {
                let scene_item_from = self.get_scene_item(*selected_scene_item_i);
                for visibility in scene_item_from.visibilities.iter() {
                    let mut previous_scene_point: ScenePoint = scene_item_from.position;
                    let mut previous_opacity: f32 = 0.0;

                    for (segment_scene_point, segment_new_opacity) in
                        visibility.opacity_segments.iter().skip(1)
                    {
                        let mut color_canal_value = 1.0 - previous_opacity;
                        if color_canal_value < 0.0 {
                            color_canal_value = 0.0;
                        }
                        mesh_builder.line(
                            &vec![previous_scene_point, *segment_scene_point],
                            1.0,
                            Color {
                                r: color_canal_value,
                                g: color_canal_value,
                                b: color_canal_value,
                                a: 1.0,
                            },
                        )?;

                        previous_scene_point = *segment_scene_point;
                        previous_opacity = *segment_new_opacity;
                    }
                }
            }
        }

        Ok(mesh_builder)
    }

    fn generate_debug_texts(&self) -> Vec<(ScenePoint, Text)> {
        let mut texts: Vec<(ScenePoint, Text)> = vec![];

        if self.debug {
            for selected_scene_item_i in self.selected_scene_items.iter() {
                let scene_item_from = self.get_scene_item(*selected_scene_item_i);
                for visibility in scene_item_from.visibilities.iter() {
                    if visibility.to_scene_item_id.is_some() {
                        texts.push((
                            visibility.to_scene_point,
                            Text::new(format!(
                                "{:.2}({:.2})",
                                visibility.path_final_opacity, visibility.to_scene_item_opacity
                            )),
                        ))
                    } else {
                        texts.push((
                            visibility.to_scene_point,
                            Text::new(format!("{:.2}", visibility.path_final_opacity)),
                        ))
                    }
                }
            }
        }

        texts
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while check_update_time(ctx, TARGET_FPS) {
            let messages = self.inputs(ctx);
            self.consume_messages(messages);

            // TODO: meta: calculer par ex qui voit qui (soldat voit un ennemi: ajouter l'event a vu
            // ennemi, dans animate il se mettra a tirer)
            let tick_sprite = self.frame_i % SPRITE_EACH == 0;
            let tick_animate = self.frame_i % ANIMATE_EACH == 0;
            let tick_seek = self.frame_i % SEEK_EACH == 0;
            let tick_physics = self.frame_i % PHYSICS_EACH == 0;
            let tick_interiors = self.frame_i % INTERIORS_EACH == 0;

            // Apply moves, explosions, etc
            if tick_physics {
                self.physics();
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
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        let mut scene_mesh_builder = MeshBuilder::new();
        let mut visibilities_mesh_builder = MeshBuilder::new();

        self.generate_scene_item_sprites()?;
        self.generate_scene_item_menu_sprites()?;

        scene_mesh_builder = self.update_scene_mesh_with_debug(scene_mesh_builder)?;
        scene_mesh_builder = self.update_scene_mesh_with_selected_items(scene_mesh_builder)?;
        scene_mesh_builder = self.update_scene_mesh_with_selection_area(scene_mesh_builder)?;
        scene_mesh_builder = self.update_scene_mesh_with_prepare_order(scene_mesh_builder)?;
        scene_mesh_builder = self.update_scene_mesh_with_projectiles(scene_mesh_builder)?;
        visibilities_mesh_builder = self.update_visibilities_mesh(visibilities_mesh_builder)?;
        let debug_texts = self.generate_debug_texts();

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

        // Draw visibilities
        if let Ok(visibilities_mesh) = visibilities_mesh_builder.build(ctx) {
            graphics::draw(ctx, &visibilities_mesh, window_draw_param)?;
        }

        // Draw debug texts
        for (text_scene_point, text) in debug_texts.iter() {
            graphics::queue_text(ctx, text, *text_scene_point, Some(graphics::WHITE));
        }
        if self.debug {
            graphics::draw_queued_text(ctx, window_draw_param, None, FilterMode::Linear)?;
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
