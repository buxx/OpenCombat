use std::cmp;
use std::collections::HashMap;

use ggez::event::MouseButton;
use ggez::graphics::{DrawMode, MeshBuilder, StrokeOptions};
use ggez::input::keyboard::KeyCode;
use ggez::timer::check_update_time;
use ggez::{event, graphics, input, Context, GameResult};

use crate::behavior::order::Order;
use crate::behavior::ItemBehavior;
use crate::config::{
    ANIMATE_EACH, DEBUG, DEFAULT_SELECTED_SQUARE_SIDE, DEFAULT_SELECTED_SQUARE_SIDE_HALF,
    DISPLAY_OFFSET_BY, DISPLAY_OFFSET_BY_SPEED, MAX_FRAME_I, META_EACH, PHYSICS_EACH,
    SCENE_ITEMS_CHANGE_ERR_MSG, SPRITE_EACH, TARGET_FPS,
};
use crate::physics::util::scene_point_from_window_point;
use crate::physics::util::window_point_from_scene_point;
use crate::physics::GridPosition;
use crate::physics::{util, MetaEvent, PhysicEvent};
use crate::scene::item::{ItemState, SceneItem, SceneItemType};
use crate::ui::scene_item_menu::SceneItemMenuItem;
use crate::ui::{SceneItemPrepareOrder, UiItem, UiSpriteInfo, UserEvent};
use crate::{Offset, ScenePoint, WindowPoint};
use std::f32::consts::FRAC_PI_2;

pub struct MainState {
    // time
    frame_i: u32,

    // display
    display_offset: Offset,
    sprite_sheet_batch: graphics::spritebatch::SpriteBatch,
    map_batch: graphics::spritebatch::SpriteBatch,
    ui_batch: graphics::spritebatch::SpriteBatch,

    // scene items
    scene_items: Vec<SceneItem>,
    scene_items_by_grid_position: HashMap<GridPosition, Vec<usize>>,

    // events
    physics_events: Vec<PhysicEvent>,

    // user interactions
    left_click_down: Option<WindowPoint>,
    right_click_down: Option<WindowPoint>,
    current_cursor_position: WindowPoint,
    user_events: Vec<UserEvent>,
    selected_scene_items: Vec<usize>,             // scene_item usize
    scene_item_menu: Option<(usize, ScenePoint)>, // scene_item usize, display_at
    scene_item_prepare_order: Option<SceneItemPrepareOrder>,
}

impl MainState {
    pub fn new(ctx: &mut Context) -> GameResult<MainState> {
        let sprite_sheet = graphics::Image::new(ctx, "/sprite_sheet.png").unwrap();
        let sprite_sheet_batch = graphics::spritebatch::SpriteBatch::new(sprite_sheet);
        let map = graphics::Image::new(ctx, "/map1bg.png").unwrap();
        let map_batch = graphics::spritebatch::SpriteBatch::new(map);
        let ui = graphics::Image::new(ctx, "/ui.png").unwrap();
        let ui_batch = graphics::spritebatch::SpriteBatch::new(ui);

        let mut scene_items = vec![];
        for x in 0..1 {
            for y in 0..4 {
                // let current_behavior = if y % 2 == 0 {
                //     ItemBehavior::WalkingTo(util::vec_from_angle(90.0))
                // } else {
                //     ItemBehavior::CrawlingTo()
                // };

                scene_items.push(SceneItem::new(
                    SceneItemType::Soldier,
                    ScenePoint::new((x as f32 * 24.0) + 100.0, (y as f32 * 24.0) + 100.0),
                    ItemState::new(ItemBehavior::Standing(0)),
                ));
            }
        }

        let mut main_state = MainState {
            frame_i: 0,
            display_offset: Offset::new(0.0, 0.0),
            sprite_sheet_batch,
            map_batch,
            ui_batch,
            scene_items,
            scene_items_by_grid_position: HashMap::new(),
            physics_events: vec![],
            left_click_down: None,
            right_click_down: None,
            current_cursor_position: WindowPoint::new(0.0, 0.0),
            user_events: vec![],
            selected_scene_items: vec![],
            scene_item_menu: None,
            scene_item_prepare_order: None,
        };

        for (i, scene_item) in main_state.scene_items.iter().enumerate() {
            let grid_position = util::grid_position_from_scene_point(&scene_item.position);
            main_state
                .scene_items_by_grid_position
                .entry(grid_position)
                .or_default()
                .push(i);
        }

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

        while let Some(user_event) = self.user_events.pop() {
            match user_event {
                UserEvent::Click(window_click_point) => self.digest_click(window_click_point),
                UserEvent::AreaSelection(window_from, window_to) => {
                    self.digest_area_selection(window_from, window_to)
                }
                UserEvent::RightClick(window_right_click_point) => {
                    self.digest_right_click(window_right_click_point)
                }
            }
        }
    }

    fn digest_click(&mut self, window_click_point: WindowPoint) {
        let scene_position =
            scene_point_from_window_point(&window_click_point, &self.display_offset);
        self.selected_scene_items.drain(..);
        if let Some(scene_item_usize) = self.get_first_scene_item_for_scene_point(&scene_position) {
            self.selected_scene_items.push(scene_item_usize);
        }

        if let Some(scene_item_prepare_order) = &self.scene_item_prepare_order {
            // TODO: Add order to scene_item
            match scene_item_prepare_order {
                SceneItemPrepareOrder::Move(scene_item_usize) => {
                    let mut scene_item = self.get_scene_item_mut(*scene_item_usize);
                    scene_item.next_order = Some(Order::MoveTo(scene_position));
                }
            }

            self.scene_item_prepare_order = None;
        }

        // FIXME BS NOW: interpreter sur quel element du menu on a click ...
        if let Some((scene_item_usize, scene_menu_point)) = self.scene_item_menu {
            let window_menu_point =
                window_point_from_scene_point(&scene_menu_point, &self.display_offset);
            let menu_sprite_info = UiSpriteInfo::from_type(UiItem::SceneItemMenu);
            let scene_item = self.get_scene_item(scene_item_usize);
            if window_click_point.x >= window_menu_point.x
                && window_click_point.x <= window_menu_point.x + menu_sprite_info.width
                && window_click_point.y >= window_menu_point.y
                && window_click_point.y <= window_menu_point.y + menu_sprite_info.height
            {
                if let Some(menu_item) = menu_sprite_info.which_item_clicked(
                    window_menu_point,
                    window_click_point,
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

    // TODO: manage errors
    fn physics(&mut self) {
        // Scene items movements
        for scene_item in self.scene_items.iter_mut() {
            match scene_item.state.current_behavior {
                ItemBehavior::WalkingTo(scene_point) => {
                    // FIXME BS NOW: velocity
                    let move_vector = (scene_point - scene_item.position).normalize() * 1.0;
                    // TODO ici il faut calculer le déplacement réél (en fonction des ticks, etc ...)
                    scene_item.position.x += move_vector.x;
                    scene_item.position.y += move_vector.y;
                    scene_item.grid_position =
                        util::grid_position_from_scene_point(&scene_item.position);
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
                        scene_item.meta_events.push(MetaEvent::FeelExplosion);
                    }
                }
            }
        }
    }

    fn animate(&mut self) {
        // TODO: ici il faut reflechir a comment organiser les comportements

        for scene_item in self.scene_items.iter_mut() {
            // for meta_event in &scene_item.meta_events {
            //     match meta_event {
            //         MetaEvent::FeelExplosion => {
            //             scene_item.state = ItemState::new(ItemBehavior::Standing(self.frame_i));
            //         }
            //     }
            // }

            // match scene_item.state.current_behavior {
            //     ItemBehavior::Crawling => {
            //         scene_item.state =
            //             ItemState::new(ItemBehavior::Walking(util::vec_from_angle(90.0)));
            //     }
            //     ItemBehavior::Walking(_) => {
            //         scene_item.state = ItemState::new(ItemBehavior::Crawling);
            //     }
            //     ItemBehavior::Standing(since) => {
            //         if self.frame_i - since >= 120 {
            //             scene_item.state =
            //                 ItemState::new(ItemBehavior::Walking(util::vec_from_angle(90.0)));
            //         }
            //     }
            // }

            scene_item.meta_events.drain(..);

            if let Some(next_order) = &scene_item.next_order {
                // TODO: Compute here if it possible (fear, compatible with current order, etc)
                match next_order {
                    Order::MoveTo(move_to_scene_point) => {
                        scene_item.current_order = Some(Order::MoveTo(*move_to_scene_point));
                    }
                }
                scene_item.next_order = None;
            }

            // FIXME BS NOW: stop move when move is accomplished; warn: recompute move_vector here
            if let Some(current_order) = &scene_item.current_order {
                match current_order {
                    Order::MoveTo(move_to_scene_point) => {
                        let change_to_walk = match scene_item.state.current_behavior {
                            ItemBehavior::Standing(_) => true,
                            ItemBehavior::CrawlingTo(_) => true,
                            ItemBehavior::WalkingTo(_) => false,
                        };

                        if change_to_walk {
                            scene_item.state =
                                ItemState::new(ItemBehavior::WalkingTo(*move_to_scene_point));
                        }
                    }
                }
            }

            match scene_item.state.current_behavior {
                ItemBehavior::Standing(_) => {}
                ItemBehavior::CrawlingTo(scene_point) => {
                    let angle = f32::atan2(
                        scene_point.y - scene_item.position.y,
                        scene_point.x - scene_item.position.x,
                    ) + FRAC_PI_2;
                    scene_item.display_angle = angle;

                    let move_vector = (scene_point - scene_item.position).normalize() * 1.0;
                    println!("{:?}", move_vector);
                }
                ItemBehavior::WalkingTo(scene_point) => {
                    let angle = f32::atan2(
                        scene_point.y - scene_item.position.y,
                        scene_point.x - scene_item.position.x,
                    ) + FRAC_PI_2;
                    scene_item.display_angle = angle;

                    let move_vector = (scene_point - scene_item.position).normalize() * 1.0;
                    println!("{:?}", move_vector);
                }
            }
        }
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
                    .as_draw_param(scene_item.current_frame as f32)
                    .dest(scene_item.position.clone()),
            );
        }

        Ok(())
    }

    fn generate_scene_item_menu_sprites(&mut self) -> GameResult {
        if let Some((_, scene_point)) = self.scene_item_menu {
            self.ui_batch.add(
                UiSpriteInfo::from_type(UiItem::SceneItemMenu)
                    .as_draw_param()
                    .dest(scene_point),
            );
        }

        Ok(())
    }

    fn generate_map_sprites(&mut self) -> GameResult {
        self.map_batch.add(
            graphics::DrawParam::new()
                .src(graphics::Rect::new(0.0, 0.0, 1.0, 1.0))
                .dest(ScenePoint::new(0.0, 0.0)),
        );

        Ok(())
    }

    fn update_mesh_builder_with_debug(
        &self,
        mut mesh_builder: MeshBuilder,
    ) -> GameResult<MeshBuilder> {
        if DEBUG {
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
                scene_point_from_window_point(&self.current_cursor_position, &self.display_offset),
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
            let selected_scene_item = self.get_scene_item(*i);
            mesh_builder.rectangle(
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
                scene_point_from_window_point(&self.current_cursor_position, &self.display_offset);
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
                SceneItemPrepareOrder::Move(scene_item_usize) => {
                    let scene_item = self.get_scene_item(*scene_item_usize);
                    mesh_builder.line(
                        &vec![
                            scene_item.position.clone(),
                            scene_point_from_window_point(
                                &self.current_cursor_position,
                                &self.display_offset,
                            ),
                        ],
                        2.0,
                        graphics::WHITE,
                    )?;
                }
            }
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

        self.generate_scene_item_sprites()?;
        self.generate_scene_item_menu_sprites()?;
        self.generate_map_sprites()?;

        scene_mesh_builder = self.update_mesh_builder_with_debug(scene_mesh_builder)?;
        scene_mesh_builder = self.update_mesh_builder_with_selected_items(scene_mesh_builder)?;
        scene_mesh_builder = self.update_mesh_builder_with_selection_area(scene_mesh_builder)?;
        scene_mesh_builder = self.update_mesh_builder_with_prepare_order(scene_mesh_builder)?;

        let scene_mesh = scene_mesh_builder.build(ctx)?;
        let window_draw_param = graphics::DrawParam::new().dest(window_point_from_scene_point(
            &ScenePoint::new(0.0, 0.0),
            &self.display_offset,
        ));

        graphics::draw(ctx, &self.map_batch, window_draw_param)?;
        graphics::draw(ctx, &self.sprite_sheet_batch, window_draw_param)?;
        graphics::draw(ctx, &scene_mesh, window_draw_param)?;
        graphics::draw(ctx, &self.ui_batch, window_draw_param)?;

        self.sprite_sheet_batch.clear();
        self.map_batch.clear();
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
        self.current_cursor_position = WindowPoint::new(x, y);
    }
}
