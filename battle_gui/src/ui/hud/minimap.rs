use battle_core::types::{Offset, SquadUuid, WindowPoint, WorldPoint};
use ggegui::egui::Vec2;
use ggez::{
    graphics::{
        Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBuilder, Rect, StrokeOptions,
    },
    Context, GameResult,
};

use crate::{graphics::qualified::Zoom, ui::component::Component};

use super::{
    battle::BATTLE_BUTTON_HEIGHT,
    builder::{MARGIN, RIGHT_BOX_WIDTH},
    event::HudEvent,
    HUD_HEIGHT,
};

pub const MINIMAP_WIDTH: f32 = RIGHT_BOX_WIDTH - MARGIN;
pub const MINIMAP_HEIGHT: f32 = HUD_HEIGHT - BATTLE_BUTTON_HEIGHT - (MARGIN * 2.);
pub const SQUAD_SQUARE_SIZE: f32 = 5.;

pub struct Minimap {
    point: WindowPoint,
    map_width: f32,
    map_height: f32,
    scene_offset: Offset,
    zoom: Zoom,
    blue_positions: Vec<WorldPoint>,
    red_positions: Vec<WorldPoint>,
    selected_squads: Vec<WorldPoint>,
}

impl Minimap {
    pub fn new(
        point: WindowPoint,
        map_width: f32,
        map_height: f32,
        scene_offset: Offset,
        zoom: Zoom,
        blue_positions: Vec<WorldPoint>,
        red_positions: Vec<WorldPoint>,
        selected_squads: Vec<WorldPoint>,
    ) -> Self {
        Self {
            point,
            map_width,
            map_height,
            scene_offset,
            zoom,
            blue_positions,
            red_positions,
            selected_squads,
        }
    }

    pub fn draw_displayed_zone(&self, ctx: &Context, mesh_builder: &mut MeshBuilder) -> GameResult {
        let (w_width, w_height) = ctx.gfx.drawable_size();
        let (w_width, w_height) = (w_width, w_height - HUD_HEIGHT);
        let world_start_x = -self.scene_offset.x;
        let world_start_y = -self.scene_offset.y;
        let world_end_x = world_start_x + w_width;
        let world_end_y = world_start_y + w_height;

        let world_start_x = world_start_x / self.zoom.factor();
        let world_start_y = world_start_y / self.zoom.factor();
        let world_end_x = world_end_x / self.zoom.factor();
        let world_end_y = world_end_y / self.zoom.factor();

        let world_start_x_rel = world_start_x / self.map_width;
        let world_start_y_rel = world_start_y / self.map_height;
        let world_end_x_rel = world_end_x / self.map_width;
        let world_end_y_rel = world_end_y / self.map_height;

        let start_x = self.point.x + (self.width(ctx) * world_start_x_rel);
        let start_y = self.point.y + (self.height(ctx) * world_start_y_rel);
        let end_x = self.point.x + (self.width(ctx) * world_end_x_rel);
        let end_y = self.point.y + (self.height(ctx) * world_end_y_rel);

        let start_x = start_x.max(self.point.x);
        let start_y = start_y.max(self.point.y);
        let end_x = end_x.min(self.point.x + self.width(ctx));
        let end_y = end_y.min(self.point.y + self.height(ctx));

        mesh_builder.rectangle(
            DrawMode::Stroke(StrokeOptions::default()),
            Rect::new(start_x, start_y, end_x - start_x, end_y - start_y),
            Color::WHITE,
        )?;

        Ok(())
    }

    pub fn draw_squads(&self, ctx: &Context, mesh_builder: &mut MeshBuilder) -> GameResult {
        for position in &self.blue_positions {
            let stroke = self.selected_squads.contains(position);
            self.draw_side_squads(ctx, mesh_builder, position, Color::BLUE, stroke)?
        }
        for position in &self.red_positions {
            self.draw_side_squads(ctx, mesh_builder, position, Color::RED, false)?
        }

        Ok(())
    }

    pub fn draw_side_squads(
        &self,
        ctx: &Context,
        mesh_builder: &mut MeshBuilder,
        position: &WorldPoint,
        color: Color,
        stroke: bool,
    ) -> GameResult {
        let relative_point =
            Vec2::new(position.x, position.y) / Vec2::new(self.map_width, self.map_height);
        let relative_point = relative_point * Vec2::new(self.width(ctx), self.height(ctx));
        let point = Vec2::new(
            self.point.x + relative_point.x,
            self.point.y + relative_point.y,
        );

        mesh_builder.rectangle(
            DrawMode::Fill(FillOptions::default()),
            Rect::new(
                point.x - SQUAD_SQUARE_SIZE / 2.,
                point.y - SQUAD_SQUARE_SIZE / 2.,
                SQUAD_SQUARE_SIZE,
                SQUAD_SQUARE_SIZE,
            ),
            color,
        )?;

        if stroke {
            mesh_builder.rectangle(
                DrawMode::Stroke(StrokeOptions::DEFAULT),
                Rect::new(
                    point.x - SQUAD_SQUARE_SIZE / 2.,
                    point.y - SQUAD_SQUARE_SIZE / 2.,
                    SQUAD_SQUARE_SIZE,
                    SQUAD_SQUARE_SIZE,
                ),
                Color::GREEN,
            )?;
        }

        Ok(())
    }
}

impl Component<HudEvent> for Minimap {
    fn point(&self, _ctx: &ggez::Context) -> WindowPoint {
        self.point
    }

    fn width(&self, _ctx: &ggez::Context) -> f32 {
        MINIMAP_WIDTH
    }

    fn height(&self, _ctx: &ggez::Context) -> f32 {
        MINIMAP_HEIGHT
    }

    fn draw(&self, ctx: &mut Context, _hovered: &WindowPoint, canvas: &mut Canvas) -> GameResult {
        let mut mesh_builder = MeshBuilder::new();

        self.draw_displayed_zone(ctx, &mut mesh_builder)?;
        self.draw_squads(ctx, &mut mesh_builder)?;

        canvas.draw(
            &Mesh::from_data(ctx, mesh_builder.build()),
            DrawParam::new(),
        );

        Ok(())
    }

    fn event(&self, ctx: &Context) -> Option<HudEvent> {
        let mouse_position = ctx.mouse.position();
        let in_x = mouse_position.x - self.point(ctx).x;
        let in_y = mouse_position.y - self.point(ctx).y;
        let relative_x = in_x / self.width(ctx);
        let relative_y = in_y / self.height(ctx);

        let world_point =
            WorldPoint::new(self.map_width * relative_x, self.map_height * relative_y);

        Some(HudEvent::CenterMapOn(world_point))
    }
}
