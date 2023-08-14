use battle_core::types::{Offset, WindowPoint, WorldPoint};
use ggez::{
    graphics::{Canvas, Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect, StrokeOptions},
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

pub struct Minimap {
    point: WindowPoint,
    map_width: f32,
    map_height: f32,
    scene_offset: Offset,
    zoom: Zoom,
}

impl Minimap {
    pub fn new(
        point: WindowPoint,
        map_width: f32,
        map_height: f32,
        scene_offset: Offset,
        zoom: Zoom,
    ) -> Self {
        Self {
            point,
            map_width,
            map_height,
            scene_offset,
            zoom,
        }
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

        let mut mesh_builder = MeshBuilder::new();
        mesh_builder.rectangle(
            DrawMode::Stroke(StrokeOptions::default()),
            Rect::new(start_x, start_y, end_x - start_x, end_y - start_y),
            Color::WHITE,
        )?;

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
