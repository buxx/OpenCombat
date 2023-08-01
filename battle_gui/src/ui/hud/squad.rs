use battle_core::{game::squad::SquadStatusesResume, types::WindowPoint};
use ggez::{
    graphics::{Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBuilder, Rect},
    Context, GameResult,
};
use glam::Vec2;
use oc_core::graphics::squad::{
    SQUAD_REL_TYPE1_HEIGHT, SQUAD_REL_TYPE1_START_X, SQUAD_REL_TYPE1_START_Y,
    SQUAD_REL_TYPE1_WIDTH, SQUAD_TYPE_WIDTH,
};

use crate::ui::component::Component;

use super::{
    builder::{BOTTOM_LINE_HEIGHT, MARGIN, RIGHT_BOX_WIDTH},
    event::HudEvent,
    HUD_HEIGHT,
};

pub const SQUAD_CARD_WIDTH: f32 = 201.;
pub const SQUAD_CARD_HEIGHT: f32 = 52.;
pub const SQUAD_CARD_MARGIN: f32 = 1.;

pub struct SquadStatuses {
    squad_statuses: SquadStatusesResume,
    point: WindowPoint,
}

struct DrawCard {
    dest: WindowPoint,
}

impl SquadStatuses {
    pub fn new(squad_statuses: SquadStatusesResume, point: WindowPoint) -> Self {
        Self {
            squad_statuses,
            point,
        }
    }

    fn cards(&self, ctx: &Context) -> Vec<DrawCard> {
        let mut draw_cards = vec![];

        let columns = (self.width(ctx) / SQUAD_CARD_WIDTH) as usize;
        for (i, _) in self.squad_statuses.squads().iter().enumerate() {
            let row_i = i / columns;
            let col_i = i % columns;
            let dest = self.point.apply(Vec2::new(
                (col_i as f32 * SQUAD_CARD_WIDTH) + SQUAD_CARD_MARGIN,
                (row_i as f32 * SQUAD_CARD_HEIGHT) + SQUAD_CARD_MARGIN,
            ));
            draw_cards.push(DrawCard { dest })
        }

        draw_cards
    }
}

impl Component<HudEvent> for SquadStatuses {
    fn point(&self, _ctx: &Context) -> WindowPoint {
        self.point
    }

    fn width(&self, ctx: &Context) -> f32 {
        ctx.gfx.drawable_size().0 - RIGHT_BOX_WIDTH - MARGIN * 2.
    }

    fn height(&self, _ctx: &Context) -> f32 {
        HUD_HEIGHT - BOTTOM_LINE_HEIGHT - MARGIN * 2.
    }

    fn sprites(&self, ctx: &Context, _hovered: &WindowPoint) -> Vec<DrawParam> {
        let mut params = vec![];
        for draw_card in self.cards(ctx) {
            // let row_i = i / columns;
            // let col_i = i % columns;
            // let dest = self.point.apply(Vec2::new(
            //     (col_i as f32 * SQUAD_CARD_WIDTH) + SQUAD_CARD_MARGIN,
            //     (row_i as f32 * SQUAD_CARD_HEIGHT) + SQUAD_CARD_MARGIN,
            // ));
            // FIXME BS NOW: According to squad type
            params.push(
                DrawParam::new()
                    .src(Rect::new(
                        SQUAD_REL_TYPE1_START_X,
                        SQUAD_REL_TYPE1_START_Y,
                        SQUAD_REL_TYPE1_WIDTH,
                        SQUAD_REL_TYPE1_HEIGHT,
                    ))
                    .dest(draw_card.dest.to_vec2()),
            )
        }

        params
    }

    fn draw(&self, ctx: &mut Context, _hovered: &WindowPoint, canvas: &mut Canvas) -> GameResult {
        let mut mesh_builder = MeshBuilder::new();

        for draw_card in self.cards(ctx) {
            mesh_builder.rectangle(
                DrawMode::Fill(FillOptions::default()),
                Rect::new(
                    draw_card.dest.x + SQUAD_TYPE_WIDTH + SQUAD_CARD_MARGIN,
                    draw_card.dest.y + SQUAD_CARD_MARGIN,
                    SQUAD_CARD_WIDTH - (SQUAD_TYPE_WIDTH + SQUAD_CARD_MARGIN * 2.),
                    12., // FIXME BS NOW : const
                ),
                Color::GREEN, // FIXME BS NOW : according to health
            )?;
        }
        canvas.draw(
            &Mesh::from_data(ctx, mesh_builder.build()),
            DrawParam::new(),
        );

        Ok(())
    }
}
