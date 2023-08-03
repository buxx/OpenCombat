use battle_core::{
    game::squad::{SquadStatusResume, SquadStatusesResume},
    types::WindowPoint,
};
use ggez::{
    graphics::{
        Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBuilder, Rect, StrokeOptions,
    },
    Context, GameResult,
};
use glam::Vec2;
use oc_core::graphics::{
    ammunition::AmmunitionReserveStatus,
    squad::{
        SQUAD_REL_TYPE1_HEIGHT, SQUAD_REL_TYPE1_START_X, SQUAD_REL_TYPE1_START_Y,
        SQUAD_REL_TYPE1_WIDTH, SQUAD_TYPE_WIDTH,
    },
};

use crate::ui::{color::Colorized, component::Component};

use super::{
    builder::{BOTTOM_LINE_HEIGHT, MARGIN, RIGHT_BOX_WIDTH},
    event::HudEvent,
    HUD_HEIGHT,
};

pub const SQUAD_CARD_WIDTH: f32 = 201.;
pub const SQUAD_CARD_HEIGHT: f32 = 52.;
pub const SQUAD_CARD_MARGIN: f32 = 1.;
pub const SQUAD_CARD_HEADER_HEIGHT: f32 = 12.;
pub const SQUAD_CARD_SOLDIER_HEALTH_WIDTH: f32 = 12.;
pub const SQUAD_CARD_SOLDIER_HEALTH_HEIGHT: f32 = 12.;

pub struct SquadStatuses {
    squad_statuses: SquadStatusesResume,
    point: WindowPoint,
}

struct DrawCard {
    dest: WindowPoint,
    squad_status: SquadStatusResume,
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
        for (i, squad_status) in self.squad_statuses.squads().into_iter().enumerate() {
            let row_i = i / columns;
            let col_i = i % columns;
            let dest = self.point.apply(Vec2::new(
                (col_i as f32 * SQUAD_CARD_WIDTH) + SQUAD_CARD_MARGIN,
                (row_i as f32 * SQUAD_CARD_HEIGHT) + SQUAD_CARD_MARGIN,
            ));
            draw_cards.push(DrawCard {
                dest,
                squad_status: squad_status.clone(),
            })
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
            );

            let soldiers_healths_start_point = draw_card.dest.apply(Vec2::new(
                SQUAD_TYPE_WIDTH + SQUAD_CARD_MARGIN,
                SQUAD_CARD_HEADER_HEIGHT + SQUAD_CARD_MARGIN,
            ));
            for (i, _) in draw_card.squad_status.members().iter().enumerate() {
                let soldiers_health_dest = soldiers_healths_start_point.apply(Vec2::new(
                    (SQUAD_CARD_SOLDIER_HEALTH_WIDTH + SQUAD_CARD_MARGIN) * i as f32,
                    0.,
                ));

                // FIXME BS NOW : Oh, fuck, it is en dessous de carre vert
                // --> Faire des images plutot que des carres vert
                // --> etat munition dessus
                // --> lisere jaune/rouge en fonction du tirs subits
                params.push(
                    DrawParam::new()
                        .src(Rect::from(AmmunitionReserveStatus::Ok.relative_src()))
                        .dest(soldiers_health_dest.to_vec2()),
                );
            }
        }

        params
    }

    fn draw(&self, ctx: &mut Context, hovered: &WindowPoint, canvas: &mut Canvas) -> GameResult {
        let mut mesh_builder = MeshBuilder::new();

        for draw_card in self.cards(ctx) {
            // Health color
            mesh_builder.rectangle(
                DrawMode::Fill(FillOptions::default()),
                Rect::new(
                    draw_card.dest.x + SQUAD_TYPE_WIDTH,
                    draw_card.dest.y,
                    SQUAD_CARD_WIDTH - (SQUAD_TYPE_WIDTH + SQUAD_CARD_MARGIN),
                    SQUAD_CARD_HEADER_HEIGHT,
                ),
                Color::new(0.5, draw_card.squad_status.health().0, 0., 1.),
            )?;

            // Soldiers healths
            let soldiers_healths_start_point = draw_card.dest.apply(Vec2::new(
                SQUAD_TYPE_WIDTH + SQUAD_CARD_MARGIN,
                SQUAD_CARD_HEADER_HEIGHT + SQUAD_CARD_MARGIN,
            ));
            for (i, solider_status) in draw_card.squad_status.members().iter().enumerate() {
                let soldiers_health_dest = soldiers_healths_start_point.apply(Vec2::new(
                    (SQUAD_CARD_SOLDIER_HEALTH_WIDTH + SQUAD_CARD_MARGIN) * i as f32,
                    0.,
                ));
                mesh_builder.rectangle(
                    DrawMode::Fill(FillOptions::default()),
                    Rect::new(
                        soldiers_health_dest.x,
                        soldiers_health_dest.y,
                        SQUAD_CARD_SOLDIER_HEALTH_WIDTH,
                        SQUAD_CARD_SOLDIER_HEALTH_HEIGHT,
                    ),
                    solider_status.health().color(),
                )?;

                // Under fire outline
                mesh_builder.rectangle(
                    DrawMode::Stroke(StrokeOptions::default()),
                    Rect::new(
                        soldiers_health_dest.x,
                        soldiers_health_dest.y,
                        SQUAD_CARD_SOLDIER_HEALTH_WIDTH,
                        SQUAD_CARD_SOLDIER_HEALTH_HEIGHT,
                    ),
                    Color {
                        r: 1.,
                        g: 0.,
                        b: 0.,
                        a: solider_status.under_fire_coefficient(),
                    },
                )?;
            }

            // Outline when hover
            let outline = Rect::new(
                draw_card.dest.x,
                draw_card.dest.y,
                SQUAD_CARD_WIDTH,
                SQUAD_CARD_HEIGHT,
            );
            if outline.contains(hovered.to_vec2()) {
                mesh_builder.rectangle(
                    DrawMode::Stroke(StrokeOptions::default()),
                    outline,
                    Color::YELLOW,
                )?;
            }
        }

        canvas.draw(
            &Mesh::from_data(ctx, mesh_builder.build()),
            DrawParam::new(),
        );

        Ok(())
    }
}
