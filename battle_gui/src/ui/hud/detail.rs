use battle_core::{game::squad::SquadStatusResume, types::WindowPoint};
use ggez::{
    graphics::{
        Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBuilder, Rect, Text,
        TextFragment, TextLayout,
    },
    Context, GameResult,
};
use glam::Vec2;
use oc_core::graphics::squad::{
    SOLDIER_HEIGHT, SOLDIER_REL_1_START_X, SOLDIER_REL_1_START_Y, SOLDIER_REL_HEIGHT,
    SOLDIER_REL_WIDTH, SOLDIER_WIDTH, SQUAD_REL_TYPE1_HEIGHT, SQUAD_REL_TYPE1_START_X,
    SQUAD_REL_TYPE1_START_Y, SQUAD_REL_TYPE1_WIDTH, SQUAD_TYPE_HEIGHT, SQUAD_TYPE_WIDTH,
};

use crate::{
    ui::{
        component::Component,
        health::{HEALTH_HEIGHT, HEALTH_WIDTH},
    },
    utils::IntoSprite,
};

use super::{event::HudEvent, HUD_HEIGHT};

pub const SQUAD_DETAIL_WIDTH: f32 = 250.;
pub const MARGIN: f32 = 1.;

pub struct SquadDetail {
    point: WindowPoint,
    squad: Option<SquadStatusResume>,
}

impl SquadDetail {
    pub fn new(point: WindowPoint, status: Option<SquadStatusResume>) -> Self {
        Self {
            point,
            squad: status,
        }
    }

    pub fn empty(point: WindowPoint) -> Self {
        Self { point, squad: None }
    }
}

impl Component<HudEvent> for SquadDetail {
    fn point(&self, _ctx: &Context) -> battle_core::types::WindowPoint {
        self.point
    }

    fn width(&self, _ctx: &Context) -> f32 {
        SQUAD_DETAIL_WIDTH
    }

    fn height(&self, _ctx: &Context) -> f32 {
        HUD_HEIGHT
    }

    fn sprites(&self, ctx: &Context, _hovered: &WindowPoint) -> Vec<DrawParam> {
        let mut params = vec![];

        if let Some(squad) = &self.squad {
            let squad_illustration_point = self.point;
            // FIXME BS NOW: According to squad type
            params.push(
                DrawParam::new()
                    .src(Rect::new(
                        SQUAD_REL_TYPE1_START_X,
                        SQUAD_REL_TYPE1_START_Y,
                        SQUAD_REL_TYPE1_WIDTH,
                        SQUAD_REL_TYPE1_HEIGHT,
                    ))
                    .dest(squad_illustration_point.to_vec2()),
            );

            let soldiers_healths_start_point =
                squad_illustration_point.apply(Vec2::new(0., SQUAD_TYPE_HEIGHT + MARGIN));
            for (i, soldier_status) in squad.members().iter().enumerate() {
                let soldier_dest = soldiers_healths_start_point
                    .apply(Vec2::new(0., (SOLDIER_HEIGHT + MARGIN) * i as f32));

                // FIXME BS NOW: According to soldier
                params.push(
                    DrawParam::new()
                        .src(Rect::new(
                            SOLDIER_REL_1_START_X,
                            SOLDIER_REL_1_START_Y,
                            SOLDIER_REL_WIDTH,
                            SOLDIER_REL_HEIGHT,
                        ))
                        .dest(soldier_dest.to_vec2()),
                );

                let soldier_health_dest = soldier_dest.apply(Vec2::new(SOLDIER_WIDTH + MARGIN, 0.));
                let soldier_health_width = self.width(ctx) - MARGIN - SOLDIER_WIDTH;
                let soldier_health_height = SOLDIER_HEIGHT;
                params.push(
                    DrawParam::new()
                        .src(Rect::from(soldier_status.health().to_relative_array()))
                        .dest(soldier_health_dest.to_vec2())
                        // We cheat by using health square image
                        // so adapt it to expected width and height
                        .scale(Vec2::new(
                            soldier_health_width / HEALTH_WIDTH,
                            soldier_health_height / HEALTH_HEIGHT,
                        )),
                );
            }
        }

        params
    }

    fn draw(&self, ctx: &mut Context, _hovered: &WindowPoint, canvas: &mut Canvas) -> GameResult {
        if let Some(squad) = &self.squad {
            let mut mesh_builder = MeshBuilder::new();
            let health_point = self.point.apply(Vec2::new(SQUAD_TYPE_WIDTH + MARGIN, 0.));

            mesh_builder.rectangle(
                DrawMode::Fill(FillOptions::default()),
                Rect::new(
                    health_point.x,
                    health_point.y,
                    self.width(ctx) - (SQUAD_TYPE_WIDTH + MARGIN * 2.),
                    SQUAD_TYPE_HEIGHT,
                ),
                Color::new(0.5, squad.health().0, 0., 1.),
            )?;

            let soldiers_status_start_point = self
                .point
                .apply(Vec2::new(SOLDIER_WIDTH, SQUAD_TYPE_HEIGHT + MARGIN));
            for (i, soldier_status) in squad.members().iter().enumerate() {
                let text_dest = soldiers_status_start_point
                    .apply(Vec2::new(0., (SOLDIER_HEIGHT + MARGIN) * i as f32));
                let text_center_dest = text_dest.apply(Vec2::new(
                    (self.width(ctx) - MARGIN - SOLDIER_WIDTH) / 2.,
                    SOLDIER_HEIGHT / 2.,
                ));
                canvas.draw(
                    Text::new(
                        TextFragment::new(soldier_status.current().display()).color(Color::WHITE),
                    )
                    .set_layout(TextLayout::center()),
                    DrawParam::default().dest(text_center_dest.to_vec2()),
                )
            }

            canvas.draw(
                &Mesh::from_data(ctx, mesh_builder.build()),
                DrawParam::new(),
            );
        }

        Ok(())
    }
}
