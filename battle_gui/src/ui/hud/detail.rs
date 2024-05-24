use battle_core::{
    game::squad::SquadStatusResume,
    types::{SoldierIndex, WindowPoint},
};
use ggez::{
    graphics::{
        Canvas, Color, DrawMode, DrawParam, FillOptions, Mesh, MeshBuilder, Rect, StrokeOptions,
        Text, TextFragment, TextLayout,
    },
    Context, GameResult,
};
use glam::Vec2;
use oc_core::graphics::squad::{
    SOLDIER_HEIGHT, SOLDIER_REL_1_START_X, SOLDIER_REL_1_START_Y, SOLDIER_REL_HEIGHT,
    SOLDIER_REL_WIDTH, SOLDIER_WIDTH, SQUAD_TYPE_HEIGHT, SQUAD_TYPE_WIDTH,
};

use crate::{
    graphics::utils::IntoDrawParam,
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
    selected_soldier: Option<SoldierIndex>,
}

impl SquadDetail {
    pub fn new(
        point: WindowPoint,
        status: Option<SquadStatusResume>,
        selected_soldier: Option<SoldierIndex>,
    ) -> Self {
        Self {
            point,
            squad: status,
            selected_soldier,
        }
    }

    pub fn empty(point: WindowPoint) -> Self {
        Self {
            point,
            squad: None,
            selected_soldier: None,
        }
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
                squad
                    .squad_type()
                    .to_draw_param()
                    .dest(squad_illustration_point.to_vec2()),
            );

            let soldiers_healths_start_point =
                squad_illustration_point.apply(Vec2::new(0., SQUAD_TYPE_HEIGHT + MARGIN));
            for (i, soldier_status) in squad.members().iter().enumerate() {
                let soldier_dest = soldiers_healths_start_point
                    .apply(Vec2::new(0., (SOLDIER_HEIGHT + MARGIN) * i as f32));

                // FIXME BS NOW: According to soldier type
                params.push(
                    soldier_status
                        .type_()
                        .to_draw_param()
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

                params.push(
                    DrawParam::new()
                        .src(Rect::from(
                            soldier_status.ammunition_reserve().relative_src(),
                        ))
                        .dest(soldier_health_dest.to_vec2()),
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
                );

                if soldier_status.leader() {
                    mesh_builder.rectangle(
                        DrawMode::Stroke(StrokeOptions::default()),
                        Rect::new(
                            text_dest.x - SOLDIER_WIDTH + 1.,
                            text_dest.y + 1.,
                            SOLDIER_WIDTH - 1.,
                            SOLDIER_WIDTH - 1.,
                        ),
                        Color::BLUE,
                    )?;
                }

                if Some(soldier_status.soldier_index()) == self.selected_soldier {
                    mesh_builder.rectangle(
                        DrawMode::Stroke(StrokeOptions::default()),
                        Rect::new(
                            text_dest.x - SOLDIER_WIDTH,
                            text_dest.y,
                            SQUAD_DETAIL_WIDTH - 1.,
                            SOLDIER_WIDTH + 1.,
                        ),
                        Color::WHITE,
                    )?;
                }
            }

            canvas.draw(
                &Mesh::from_data(ctx, mesh_builder.build()),
                DrawParam::new(),
            );
        }

        Ok(())
    }

    fn event(&self, ctx: &Context) -> Option<HudEvent> {
        let mouse_position = ctx.mouse.position();

        if let Some(squad) = &self.squad {
            let soldiers_status_start_point = self
                .point
                .apply(Vec2::new(SOLDIER_WIDTH, SQUAD_TYPE_HEIGHT + MARGIN));
            for (i, soldier_status) in squad.members().iter().enumerate() {
                let point = soldiers_status_start_point
                    .apply(Vec2::new(0., (SOLDIER_HEIGHT + MARGIN) * i as f32));

                if mouse_position.x >= point.x
                    && mouse_position.x <= point.x + SQUAD_DETAIL_WIDTH
                    && mouse_position.y >= point.y
                    && mouse_position.y <= point.y + SOLDIER_WIDTH
                {
                    return Some(HudEvent::SelectSoldier(soldier_status.soldier_index()));
                }
            }
        }

        None
    }
}
